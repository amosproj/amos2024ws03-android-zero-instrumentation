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
    val featureName: String,
    val featureType: FeatureType,
    val active: Boolean,
) {
    data class VfsWriteOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions("VFS Write Analysis", FeatureType.IO, enabled)

    data class SendMessageOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions("Unix Domain Socket Analysis", FeatureType.IO, enabled)

    data class JniReferencesOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions("Local & Global Indirect JNI References", FeatureType.IO, enabled)

    data class SigquitOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions("SIGQUIT", FeatureType.SIGNALS, enabled)

    data class UprobeOption(
        val method: String,
        val enabled: Boolean,
        val pids: Set<UInt>,
        val offset: ULong,
        val odexFilePath: String,
    ) : BackendFeatureOptions(method, FeatureType.UPROBES, enabled)
}
