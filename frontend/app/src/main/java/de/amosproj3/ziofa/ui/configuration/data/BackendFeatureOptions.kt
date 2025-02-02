// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

enum class FeatureType(val displayName: String) {
    IO("IO Observability"),
    SIGNALS("Linux Signals"),
    MEMORY("Memory Usage"),
    UPROBES("Uprobes"),
}

/** Central wrapper for backend features and their properties.
 * This class is widely used and is converted into from classes from the Client SDK, as well as the
 * other way around.
 * It amends the Client SDK configurations with additional information relevant for the UI,
 * such as the display name and the type of the feature. (for grouping)
 * When updating the properties of a Client SDk configuration, this class will likely have to be
 * updated as well.
 * */
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
            type = FeatureType.MEMORY,
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

    data class GcOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions(
            name = "Garbage Collector Analysis & Heap Usage",
            type = FeatureType.MEMORY,
            description = "View live GC invocations, used Java heap and total Java heap size.",
            active = enabled,
        )

    data class OpenFileDescriptors(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions(
            name = "Open File Descriptors",
            type = FeatureType.IO,
            description = "View the number of opened file descriptors.",
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
