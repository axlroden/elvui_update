
# elvui_update
A script for automatically updating ELVUI SoD/Classic/Retail when a new version is released.

## What it does:
This program will query the tukui API to determine if the local version matches the current release. The new version will be downloaded and installed if there is a version mismatch.
It finds the WoW folders automatically.
Added output of changelog to console.

Windows Defender does not like this, turn off real time protection, and allow the file once you put it on your desktop. You can see the source and build script here on github, so nothing to worry about.

## Install the exe by scheduling a task:
###### Windows
1. If Task Scheduler is not open, start Task Scheduler. For more information, see Start Task Scheduler.
2. Find and click the task folder in the console tree that you want to create the task in. For more information about how to create the task in a new task folder, see Create a New Task Folder.
3. In the Actions Pane, click Create Basic Task.
4. Follow the instructions in the Create Basic Task Wizard.
    
Instructions from: [MS Technet](https://technet.microsoft.com/en-us/library/cc748993(v=ws.11).aspx#BKMK_winui)
