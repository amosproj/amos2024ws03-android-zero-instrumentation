// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

enum class FeatureType(val displayName: String) {
    IO("IO Observability Features"),
    SIGNALS("Linux Signals"),
    UPROBES("Uprobes"),
}

sealed class BackendFeatureOptions(
    val name: String,
    val type: FeatureType,
    val description: String,
    val active: Boolean,
) {
    data class VfsWriteOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions(
            name = "VFS Write Analysis",
            type = FeatureType.IO,
            description = "Analyse writes to flash storage by tracing vfs_write calls.",
            active = enabled,
        )

    data class SendMessageOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions(
            name = "Unix Domain Socket Analysis",
            type = FeatureType.IO,
            description = "Analyse unix domain socket traffic by observing sys_sendmsg calls.",
            active = enabled,
        )

    data class JniReferencesOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions(
            name = "Local & Global Indirect JNI References",
            type = FeatureType.IO,
            description =
                "Detect JNI memory leaks by tracing the number of indirect JNI references.",
            active = enabled,
        )

    data class SigquitOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions(
            name = "SIGQUIT",
            type = FeatureType.SIGNALS,
            description =
                "Trace SIGQUIT signals to processes. Useful for detecting killed background processes.",
            active = enabled,
        )

    data class UprobeOption(
        val method: String,
        val enabled: Boolean,
        val pids: Set<UInt>,
        val offset: ULong,
        val odexFilePath: String,
    ) :
        BackendFeatureOptions(
            name = method,
            type = FeatureType.UPROBES,
            description = "Tracking calls to $method",
            active = enabled,
        )
}
