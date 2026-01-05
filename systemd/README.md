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
     Active: active (running) since Thu 2025-07-31 10:22:09 CEST; 6s ago
   Main PID: 137204 (bash)
      Tasks: 8 (limit: 9573)
        CPU: 360ms
     CGroup: /system.slice/mydeployer.service
             ├─137204 /bin/bash /home/petste/MyDeployer/start.sh
             └─137205 /home/petste/MyDeployer/deployer --config=/home/petste/MyDeployer/config/config.toml

Jul 31 10:22:09 mygrid systemd[1]: Started mydeployer.service - Deployer accepting GitHub webhooks deploy actions.
```

If the application for some reason prints anything to stdout/stderr, such in case of a panic,
the log for that can be found by using `journalctl -u mydeployer.service`.