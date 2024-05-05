// LineInFile module : manipulate lines in a file (add, delete)

use serde::{Serialize, Deserialize};
use crate::workflow::change::ModuleBlockChange;
use crate::workflow::result::{ApiCallResult, ApiCallStatus};
use crate::modules::{DryRun, Apply};
use crate::modules::ModuleApiCall;
use connection::prelude::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineInFileBlockExpectedState {
    filepath: String,
    line: Option<String>,
    state: Option<String>,
    position: Option<String>, // "top" | "bottom" | "anywhere" (default) | "45" (specific line number)

    // ****** To be implemented ********
    // beforeline: Option<String>, // Insert before this line
    // afterline: Option<String>, // Insert after this line
    // replace: Option<String>, // Replace this line...
    // with: Option<String> // ... with this one.
}

impl DryRun for LineInFileBlockExpectedState {
    fn dry_run_block(&self, hosthandler: &mut HostHandler, privilege: Privilege) -> ModuleBlockChange {
        assert!(hosthandler.ssh2.sshsession.authenticated());

        if ! hosthandler.is_this_cmd_available("sed").unwrap() {
            return ModuleBlockChange::failed_to_evaluate("Sed command not available on this host");
        }

        let file_exists_check = hosthandler.run_cmd(
            format!("test -f {}", self.filepath).as_str(),
            privilege.clone()
        ).unwrap();
    
        if file_exists_check.exitcode != 0 {
            return ModuleBlockChange::failed_to_evaluate(
                format!("{} not found or not a regular file", self.filepath).as_str()
            );
        }

        let mut changes: Vec<ModuleApiCall> = Vec::new();
       
        match &self.state {
            Some(state) => {
                let change = match state.as_str() {
                    "present" => {
                        assert!(hosthandler.ssh2.sshsession.authenticated());

                        let mut bottom = false;
                        let filenumberoflines = hosthandler.run_cmd(
                            format!("cat {} | wc -l", self.filepath).as_str(),
                            privilege.clone()
                        ).unwrap().stdout.trim().parse::<u32>().unwrap();

                        // Parse the position attribute (where the line is expected to be)
                        let expected_position: Option<u32> = match &self.position {
                            Some(value) => {
                                match value.as_str() {
                                    "top" => { Some(1u32) }
                                    "bottom" => {
                                        bottom = true;
                                        Some(filenumberoflines)
                                    }
                                    "anywhere" => { None } // Default
                                    _ => {
                                        // Try parsing as a u32
                                        match value.parse::<u32>() {
                                            Ok(linenumber) => {
                                                if linenumber <= filenumberoflines {
                                                    Some(linenumber)
                                                } else {
                                                    return ModuleBlockChange::failed_to_evaluate("Position value out of range (use \"bottom\" instead)");
                                                }
                                                
                                            }
                                            Err(e) => { 
                                                return ModuleBlockChange::failed_to_evaluate(format!("Failed to parse position value : {}", e).as_str());
                                             }
                                        }
                                    }
                                }
                            }
                            None => { None } // Default = "anywhere" = bottom if we need to create the line
                        };

                        let file_actual_state = is_line_present(hosthandler, &self.line.as_ref().unwrap(), &self.filepath, &privilege);

                        match file_actual_state {
                            Some(actual_line_numbers) => {
                                // Line is already there but we need to make sure it is at the expected place
                                match expected_position {
                                    Some(expected_linenumber) => {
                                        if actual_line_numbers.contains(&expected_linenumber) {
                                            // Line is already at the right place, nothing to do
                                            ModuleApiCall::None(
                                                String::from("Line already present at expected place")
                                            )
                                        } else {
                                            // Line is not at the expected place and needs to be added
                                            if bottom {
                                                ModuleApiCall::LineInFile(
                                                    LineInFileApiCall {
                                                        action: "add".to_string(),
                                                        line: self.line.as_ref().unwrap().clone(),
                                                        line_numbers: None,
                                                        position: None,
                                                        path: self.filepath.clone(),
                                                        privilege
                                                    }
                                                )
                                            } else {
                                                ModuleApiCall::LineInFile(
                                                    LineInFileApiCall {
                                                        action: "add".to_string(),
                                                        line: self.line.as_ref().unwrap().clone(),
                                                        line_numbers: None,
                                                        position: expected_position,
                                                        path: self.filepath.clone(),
                                                        privilege
                                                    }
                                                )
                                            }
                                        }
                                    }
                                    None => {
                                        // Line is already present but position is not specified (aka "anywhere"), nothing to do
                                        ModuleApiCall::None(
                                            format!("Line already present {:?}", actual_line_numbers)
                                        )
                                    }
                                }
                            }
                            None => {
                                // Line is absent and needs to be added
                                if bottom {
                                    ModuleApiCall::LineInFile(
                                        LineInFileApiCall {
                                            action: "add".to_string(),
                                            line: self.line.as_ref().unwrap().clone(),
                                            line_numbers: None,
                                            position: None,
                                            path: self.filepath.clone(),
                                            privilege
                                        }
                                    )
                                } else {
                                    ModuleApiCall::LineInFile(
                                        LineInFileApiCall {
                                            action: "add".to_string(),
                                            line: self.line.as_ref().unwrap().clone(),
                                            line_numbers: None,
                                            position: expected_position,
                                            path: self.filepath.clone(),
                                            privilege
                                        }
                                    )
                                }
                            }
                        }
                    }
                    "absent" => {
                        assert!(hosthandler.ssh2.sshsession.authenticated());
                
                        // Check if line is already present
                        match is_line_present(hosthandler, self.line.as_ref().unwrap(), &self.filepath, &privilege) {
                            Some(line_numbers) => {
                                ModuleApiCall::LineInFile(
                                        LineInFileApiCall {
                                            action: "del".to_string(),
                                            line: self.line.as_ref().unwrap().clone(),
                                            line_numbers: Some(line_numbers),
                                            position: None,
                                            path: self.filepath.clone(),
                                            privilege
                                        }
                                    )
                            }
                            None => {
                                // Line is already absent
                                ModuleApiCall::None(
                                        String::from("Line already absent")
                                    )
                            }
                        }
                    }
                    _ => {
                        ModuleApiCall::None(
                            String::from("Wrong state value")
                        )
                    }
                };
                changes.push(change);
            }
            None => {}
        }

        return ModuleBlockChange::changes(changes);
    }

}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineInFileApiCall {
    path: String,
    line: String,
    line_numbers: Option<Vec<u32>>,
    position: Option<u32>, // Where to put the line in case of add
    action: String,
    privilege: Privilege
}

impl Apply for LineInFileApiCall {

    fn display(&self) -> String {
        match self.action.as_str() {
            "add" => {
                return String::from("Line missing -> needs to be added");
            }
            "del" => {
                return format!("Line present {:?} -> needs to be removed", self.line_numbers.as_ref().unwrap());
            }
            _ => { return String::from("Wrong LineInFileCall action"); }
        }
    }

    fn apply_moduleblock_change(&self, hosthandler: &mut HostHandler) -> ApiCallResult {
        assert!(hosthandler.ssh2.sshsession.authenticated());

        match self.action.as_str() {
            "add" => {
                let mut cmd = String::new();

                match self.position {
                    Some(linenumber) => {
                        // If the file is empty, the sed command won't work.
                        let filesizecheck_cmd = format!("test -s {}", self.path);
                        let filesizecheck = hosthandler.run_cmd(filesizecheck_cmd.as_str(), self.privilege.clone()).unwrap();
                        if filesizecheck.exitcode == 0 { // File not empty
                            cmd = format!("sed -i \'{} i {}\' {}", linenumber, self.line, self.path);
                        } else { // File empty
                            if linenumber == 1 { // Position = "top"
                                cmd = format!("echo \'{}\' >> {}", self.line, self.path);
                            } else { // Position = <any other value> which is out of range anyway
                                return ApiCallResult::from(
                                    Some(filesizecheck.exitcode),
                                    Some(filesizecheck.stdout),
                                    ApiCallStatus::Failure(
                                        String::from("Position value out of range (use \"bottom\" instead)")
                                    )
                                );
                            }
                        }
                        
                    }
                    None => {
                        // If no line number is specified, the default behavior is to add the line at the bottom of the file
                        cmd = format!("echo \'{}\' >> {}", self.line, self.path);
                    }
                }

                
                let cmd_result = hosthandler.run_cmd(cmd.as_str(), self.privilege.clone()).unwrap();
                
                if cmd_result.exitcode == 0 {
                    return ApiCallResult::from(
                        Some(cmd_result.exitcode),
                        Some(cmd_result.stdout),
                        ApiCallStatus::ChangeSuccessful(
                            String::from("Line added")
                        )
                    );
                } else {
                    return ApiCallResult::from(
                        Some(cmd_result.exitcode),
                        Some(cmd_result.stdout),
                        ApiCallStatus::Failure(
                            String::from("Failed to add line")
                        )
                    );
                }
            }
            "del" => {
                // We need a final command like this : sed -i '7d;12d;16d' input.txt
                // It implies a little formatting first.
                let formatted_line_numbers = self.line_numbers.clone().unwrap().into_iter().map(|i| format!("{}d;", i)).collect::<String>();
                let formatted_line_numbers = formatted_line_numbers.split_at(formatted_line_numbers.len() - 1).0; // Delete the last ';

                let cmd = format!("sed -i \'{}\' {}", formatted_line_numbers, self.path);
                let cmd_result = hosthandler.run_cmd(cmd.as_str(), self.privilege.clone()).unwrap();
                
                if cmd_result.exitcode == 0 {
                    return ApiCallResult::from(
                        Some(cmd_result.exitcode),
                        Some(cmd_result.stdout),
                        ApiCallStatus::ChangeSuccessful(
                            format!("Line {:?} removed", self.line_numbers.as_ref().unwrap())
                        )
                    );
                } else {
                    return ApiCallResult::from(
                        Some(cmd_result.exitcode),
                        Some(cmd_result.stdout),
                        ApiCallStatus::Failure(
                            String::from("Failed to remove line")
                        )
                    );
                }
            }
            _ => { return ApiCallResult::none(); }
        }
    }

}

// Returns a Some(Vec<u32>) representing the line numbers of each occurrence of the line if present, and None if absent
fn is_line_present(hosthandler: &mut HostHandler, line: &String, filepath: &String, privilege: &Privilege) -> Option<Vec<u32>> {
    let test = hosthandler.run_cmd(
        format!("grep -n -F -w \'{}\' {}", line, filepath).as_str(), //  Output looks like 4:my line content
        privilege.clone()
    ).unwrap();

    if test.exitcode == 0 {
        let mut line_numbers: Vec<u32> = Vec::new();
        for line in test.stdout.lines() {
            line_numbers.push(line.split(':').next().unwrap().parse::<u32>().unwrap());
        }
        return Some(line_numbers);
    } else {
        return None;
    }
}