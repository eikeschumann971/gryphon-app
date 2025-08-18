Please create a Rust application for me. 
The project is implemented in the DDD architecture. A domain is designed following the "ports and adapter" design.
The project uses event sourcing. It uses the latest release of for it. It uses Kafka as event store. Use a Kafka version from 3.3+ or 4.0+ so it sure that KRaft is used.
Also use Postgres for snapshots.
Create a directory directory structure and use the given layout below as a guideline.
The domains to be created are called: LogicalAgent, TechnicalAgent, KinematicAgent, PathPlanning, Dynamics, GUI 

```
library-system/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── domain1/
│   │   └── domain1.rs
│   ├── application/
│   │   └── add_domain1.rs
│   ├── resources/
│   │   └── graph.toml
│   ├── adapters/
│   │   ├── inbound/
│   │   │   ├── snapshot_store.rs 
│   │   │   └── event_store.rs
│   │   └── outbound/
│   │   │   ├── kafka.rs 
│   │   │   └── postgres.rs
│   ├── config.rs
│   └── common/
│       └── common_code.rs
├── tests/
│   ├── domain_tests.rs
│   ├── application_tests.rs
│   └── integration_tests.rs
├── docs
└── prompts
```

In addition to the example folder layout create also folders inside a domain folder that describe domain or implementation specific entities like: aggregate, events, projection actor, event actor. Some parts might  be common code i.e. code that is shared with each domain or belongs to the adapter folder.  

