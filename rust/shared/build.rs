// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use std::sync::LazyLock;

use tonic_build::Builder;

static UNIFFI_RECORDS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    if cfg!(feature = "uniffi") {
        /*
         * List here all protobuf messages to be exported. Enums will be
         * exported as int32. Oneofs ,ist be specified below.
         */
        vec![
            // ziofa.proto
            "ProcessList",
            "Process",
            "CmdlineData",
            "SearchSymbolsRequest",
            "SearchSymbolsResponse",
            "Symbol",
            "GetSymbolOffsetRequest",
            "GetSymbolOffsetResponse",

            // config.proto
            "Configuration",
            "WriteConfig",
            "BlockingConfig",
            "JniReferencesConfig",
            "SignalConfig",
            "UprobeConfig",
            "GarbageCollectConfig",
            "FileDescriptorChangeConfig",
            "StringFilter",
            "UInt32Filter",
            "Filter",
            
            // events.proto
            "Event",
            "EventContext",
            "TimeSeriesEvent",
            "TimeSeriesData",
            "LogEvent",
            "WriteEvent",
            "BlockingEvent",
            "JniReferencesEvent",
            "SignalEvent",
            "GarbageCollectEvent",
            "FileDescriptorChangeEvent",
            
            "Duration",
            "Timestamp",
        ]
    } else {
        vec![]
    }
});

static UNIFFI_ENUMS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    if cfg!(feature = "uniffi") {
        /*
         * List here all protobuf oneofs to be exported.
         */
        vec![
            "Process.cmd",
            "MissingBehavior",
            "Event.event_data",
            "LogEvent.log_event_data",
            "JniMethodName",
            "FileDescriptorOp",
            ]
    } else {
        vec![]
    }
});

fn derive_records(mut builder: Builder) -> Builder {
    for record in UNIFFI_RECORDS.iter() {
        builder = builder.type_attribute(record, "#[derive(uniffi::Record)]");
    }
    builder
}

fn derive_enums(mut builder: Builder) -> Builder {
    for record in UNIFFI_ENUMS.iter() {
        builder = builder.type_attribute(record, "#[derive(uniffi::Enum)]");
    }
    builder
}

fn main() {
    let mut builder = tonic_build::configure();

    builder = builder
        .protoc_arg("--experimental_allow_proto3_optional")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_well_known_types(true);

    builder = derive_records(builder);
    builder = derive_enums(builder);

    builder
        .compile_protos(
            /*
             * List all proto files here.
             */
            &[
                "./proto/ziofa.proto",
                "./proto/config.proto",
                "./proto/events.proto",
                "./proto/processes.proto",
                "./proto/symbols.proto",
            ],
            &["./proto"],
        )
        .unwrap();
}
