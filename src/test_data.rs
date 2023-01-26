pub const SAMPLE_LOG: &'static str = r#"[2022-12-01 15:23:01] [INFO] FictitiousApp started
[2022-12-01 15:23:02] [DEBUG] Connecting to database...
[2022-12-01 15:23:03] [DEBUG] Connection established
[2022-12-01 15:23:03] [DEBUG] Loading configuration file...
[2022-12-01 15:23:04] [INFO] Configuration loaded
[2022-12-01 15:23:04] [INFO] Starting server on localhost:8080
[2022-12-01 15:23:05] [INFO] Server started
[2022-12-01 15:24:01] [INFO] User "johndoe" logged in
[2022-12-01 15:25:01] [WARNING] User "johndoe" attempted to access unauthorized resource
[2022-12-01 15:26:01] [ERROR] Internal server error: NullPointerException
[2022-12-01 15:27:01] [INFO] User "janedoe" logged in
[2022-12-01 15:28:01] [INFO] User "janedoe" accessed resource "/profile"
[2022-12-01 15:29:01] [INFO] User "janedoe" accessed resource "/settings"
[2022-12-01 15:30:01] [INFO] User "janedoe" logged out
[2022-12-01 15:31:01] [INFO] User "testuser" logged in
[2022-12-01 15:32:01] [INFO] User "testuser" accessed resource "/dashboard"
[2022-12-01 15:33:01] [WARNING] User "testuser" has high number of failed login attempts
[2022-12-01 15:34:01] [INFO] User "testuser" logged out
[2022-12-01 15:35:01] [INFO] FictitiousApp shutting down
[2022-12-01 15:35:02] [DEBUG] Disconnecting from database...
[2022-12-01 15:35:03] [DEBUG] Connection closed
[2022-12-01 15:35:03] [INFO] FictitiousApp stopped
"#;
