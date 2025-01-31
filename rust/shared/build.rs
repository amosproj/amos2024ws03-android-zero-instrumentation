// SPDX-FileCopyrightText: 2024 Benedikt Zinn <benedikt.wh.zinn@gmail.com>
// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
// SPDX-FileCopyrightText: 2024 Tom Weisshuhn <tom.weisshuhn@fau.de>
//
// SPDX-License-Identifier: MIT

use std::sync::LazyLock;

use tonic_build::Builder;

static UNIFFI_RECORDS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    if cfg!(feature = "uniffi") {
        vec![
            "Process",
            "CmdlineData",
            "Configuration",
            "EbpfEntry",
            "UprobeConfig",
            "Event",
            "TimeSeriesEvent",
            "LogEvent",
            "VfsWriteEvent",
            "SysSendmsgEvent",
            "JniReferencesEvent",
            "SysSigquitEvent",
            "VfsWriteConfig",
            "SysSendmsgConfig",
            "JniReferencesConfig",
            "StringResponse",
            "Symbol",
            "SetConfigurationResponse",
            "SysSigquitConfig",
            "GcConfig",
            "GcEvent",
            "SysFdTrackingConfig",
            "SysFdTrackingEvent",
            "TimeSeriesType"
        ]
    } else {
        vec![]
    }
});

static UNIFFI_ENUMS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    if cfg!(feature = "uniffi") {
        vec!["Process.cmd", "Event.event_data", "JniReferencesEvent.JniMethodName", "Event.event_type", "LogEvent.event_data", "SysFdTrackingEvent.SysFdAction"]
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
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    builder = derive_records(builder);
    builder = derive_enums(builder);

    builder
        .compile_protos(
            &[
                "./proto/ziofa.proto",
                "./proto/config.proto",
                "./proto/events.proto",
            ],
            &["./proto"],
        )
        .unwrap();
}
