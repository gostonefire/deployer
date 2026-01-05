# Configure Systemd
* Check paths in `start.sh` and `mydeployer.service`
* Copy `mydeployer.service` to `/lib/systemd/system/`
* Run `sudo systemctl enable mydeployer.service`
* Run `sudo systemctl start mydeployer.service`
* Check status by running `sudo systemctl status mydeployer.service`

Output should be something like:
```
● mydeployer.service - Deployer accepting GitHub webhooks deploy actions
     Loaded: loaded (/lib/systemd/system/mydeployer.service; enabled; preset: enabled)
     Active: active (running) since Mon 2026-01-05 12:35:26 CET; 21s ago
   Main PID: 888096 (bash)
      Tasks: 7 (limit: 9568)
        CPU: 81ms
     CGroup: /system.slice/mydeployer.service
             ├─888096 /bin/bash /home/petste/MyDeployer/start.sh
             └─888097 /home/petste/MyDeployer/deployer --config=/home/petste/MyDeployer/config/config.toml

Jan 05 12:35:26 mygrid systemd[1]: Started mydeployer.service - Deployer accepting GitHub webhooks deploy actions.
```

If the application for some reason prints anything to stdout/stderr, such in case of a panic,
the log for that can be found by using `journalctl -u mydeployer.service`.