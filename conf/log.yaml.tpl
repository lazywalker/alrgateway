refresh_rate: 10 seconds
appenders:

  stdout:
    kind: console
    encoder:
      pattern: "{d} [{P}] {h({l:<5.5})} {t} - {m}{n}"

  file:
    kind: file
    path: "log/alrgateway.log"
    encoder:
      pattern: "{d} [{P}] {h({l:<5.5})} {t} - {m}{n}"

  rfile:
    kind: rolling_file
    path: "log/alrgateway.log"
    # Specifies if the appender should append to or truncate the log file if it
    # already exists. Defaults to `true`.
    append: true
    # The encoder to use to format output. Defaults to `kind: pattern`.
    encoder:
      pattern: "{d} [{P}] {h({l:<5.5})} {t} - {m}{n}"
    # The policy which handles rotation of the log file. Required.
    policy:
      # Identifies which policy is to be used. If no kind is specified, it will
      # default to "compound".
      kind: compound
      # The remainder of the configuration is passed along to the policy's
      # deserializer, and will vary based on the kind of policy.
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: delete
  sql:
    kind: file
    path: "log/sql.log"
    encoder:
      pattern: "{d} - {m}{n}"
      
  hyper:
    kind: file
    path: "log/hyper.log"
    encoder:
      pattern: "{d} - {m}{n}"

root:
  level: debug
  appenders:
    - stdout
    - rfile

loggers:
  tokio_postgres:
    level: debug
    appenders:
      - stdout
      - sql
  hyper:
    level: info
    appenders:
      - hyper
    additive: false