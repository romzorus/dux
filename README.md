# Dux : a scalable and cloud-friendly automation tool
<div align="center">
<img src="img/dux.png " width="25%">
</div>

# Our goal
Build a scalable automation / orchestration tool (something that runs a task on a remote host basically) ready to handle a massive amount of remote hosts in parallel. To increase the number of remote hosts you can handle in parallel, just increase the number of worker nodes or their capacity.

A worker node can be either a physical/virtual machine or a container.

# All-in-one dux tool

**Usage** : ```dux -t <tasklist.yaml> -l <hostlist.yaml> -k <SSH private key>```

*with `tasklist.yaml`*
~~~
- name: Prerequisites
  steps:
    - name: 1. Test SSH connectivity
      ping:

    - name: 2. Install git for Debian
      with_sudo: true
      apt:
        state: present
        package: git
        upgrade: true
~~~
*and `hostlist.yaml`*
~~~
hosts:
  - 192.168.1.6
  - 192.168.1.85
~~~
**Output example**

~~~

    ██████╗ ██╗   ██╗██╗  ██╗
    ██╔══██╗██║   ██║╚██╗██╔╝
    ██║  ██║██║   ██║ ╚███╔╝ 
    ██║  ██║██║   ██║ ██╔██╗ 
    ██████╔╝╚██████╔╝██╔╝ ██╗
    ╚═════╝  ╚═════╝ ╚═╝  ╚═╝

Host : 192.168.1.6 (Changed)
┌─────────────┬─────────────────────────┬───────────────────────────────────────┬────────────────────────────────────┐
│         Task│          Step           │                Changes                │Results                             │
├─────────────┼─────────────────────────┼───────────────────────────────────────┼────────────────────────────────────┤
│Prerequisites│1. Test SSH connectiviy  │Check SSH connectivity with remote host│Success : Host reachable through SSH│
├─────────────┼─────────────────────────┼───────────────────────────────────────┼────────────────────────────────────┤
│Prerequisites│2. Install git for Debian│             Install - git             │Success : git install successful    │
│             │                         │                Upgrade                │Success : APT upgrade successful    │
└─────────────┴─────────────────────────┴───────────────────────────────────────┴────────────────────────────────────┘

Host : 192.168.1.85 (Changed)
┌─────────────┬─────────────────────────┬───────────────────────────────────────┬────────────────────────────────────┐
│         Task│          Step           │                Changes                │Results                             │
├─────────────┼─────────────────────────┼───────────────────────────────────────┼────────────────────────────────────┤
│Prerequisites│1. Test SSH connectiviy  │Check SSH connectivity with remote host│Success : Host reachable through SSH│
├─────────────┼─────────────────────────┼───────────────────────────────────────┼────────────────────────────────────┤
│Prerequisites│2. Install git for Debian│             Install - git             │Success : git install successful    │
│             │                         │                Upgrade                │Success : APT upgrade successful    │
└─────────────┴─────────────────────────┴───────────────────────────────────────┴────────────────────────────────────┘
~~~

# Modules available
*(alphabetized)*
| Module | Description |
| ---      | ---      |
| `apt`   | Manage packages on Debian-like distributions |
| `dnf` | Manage packages on Fedora-like distributions (no difference with `yum`) |
| `ping`   | Test SSH connectivity with remote host |
| `yum` | Manage packages on Fedora-like distributions (no difference with `dnf`) |

# Have a remote host handled by Dux

## On controlled host
As for other automation tools, Dux needs an account to use on the controlled host. Let's create a `dux` user on the controlled host and give it some privileges :
```
# Create user (set password interactively)
sudo adduser dux

# Add user to sudo group
sudo usermod -aG sudo dux

# Create a sudoers file for this user
echo "dux ALL = (root) NOPASSWD:ALL" | sudo tee /etc/sudoers.d/dux
```

## On controller host (where Dux is installed)
The ideal is to have a SSH passwordless connection :
```
# Generate a SSH key (no passphrase for the example)
ssh-keygen -t ed25519 -f controller_key -N "" -q

# Have this key allowed on the controlled host
ssh-copy-id -i controller_key.pub dux@<controlled host address>
```

**After this, you can use `dux` like this from the master host**

```dux -t <tasklist.yaml> -l <hostlist.yaml> -u dux -k <path to controller_key>```


## License
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
