use std::process::exit;
use taskexec::workflow::task::TaskList;
use hostparser::Host;
use crate::fileformats::json::json_tasklist_parser;
use crate::fileformats::yaml::yaml_tasklist_parser;
use errors::Error;
use errors::FAILURE_TO_OPEN_FILE;
use errors::FAILURE_TO_PARSE_FILE;

pub fn tasklist_parser(tasklistcontent: String, host: &Host) -> TaskList {

    match yaml_tasklist_parser(&tasklistcontent, host) {
        Ok(mut parsed_content) => {

            for (taskindex, taskcontent) in parsed_content.clone().tasks.iter().enumerate() {
                for (stepindex, _stepcontent) in taskcontent.steps.iter().enumerate() {
                    let _ = parsed_content.tasks[taskindex].steps[stepindex].parsemodule(); // TODO : error handling required here
                }
            }
            return parsed_content;
        }
        Err(e) => {
            println!("Unable to parse TaskList as YAML: {:?}", e);
            println!("Trying to parse TaskList as JSON.");
            match json_tasklist_parser(&tasklistcontent, host) {
                Ok(mut parsed_content) => {
                    for (taskindex, taskcontent) in parsed_content.clone().tasks.iter().enumerate() {
                        for (stepindex, _stepcontent) in taskcontent.steps.iter().enumerate() {
                            let _ = parsed_content.tasks[taskindex].steps[stepindex].parsemodule(); // TODO : error handling required here
                        }
                    }
                    return parsed_content;
                }
                Err(e) => {
                    println!("Unable to parse TaskList JSON: {:?}", e);
                    println!("Unable to parse TaskList at all. Abort.");
                    exit(FAILURE_TO_PARSE_FILE);
                }
            }
        }
    }

}

pub fn tasklist_get_from_file(file_path: &str) -> String {
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            return content;
        }
        Err(e) => {
            println!("Unable to open TaskList file : {:?}", e);
            exit(FAILURE_TO_OPEN_FILE);
        }
    }
}

pub fn tasklist_get_from_interactive_mode() -> String {
    // Placeholder : interactive_mode is when the final user sets a group of hosts and can
    // type commands as in a normal shell interpreter but each command is directly turned
    // into a Task, executed on the group of hosts, and the results arrive in "realtime".
    String::new()
}

// Will be usefull in the future
// pub enum ContentFormat {
//     Interactive,
//     Json,
//     Pythonscript,
//     Shellscript,
//     Yaml
// }
