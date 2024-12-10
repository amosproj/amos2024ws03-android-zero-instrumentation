// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

sealed class BackendFeatureOptions(val featureName: String, val active: Boolean) {
    data class VfsWriteOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions("VFS Write Analysis", enabled)

    data class SendMessageOption(val enabled: Boolean, val pids: Set<UInt>) :
        BackendFeatureOptions("Unix Domain Socket Analysis", enabled)

    data class UprobeOption(
        val method: String,
        val enabled: Boolean,
        val pids: Set<UInt>,
        val offset: ULong,
        val odexFilePath: String,
    ) : BackendFeatureOptions(method, enabled)
}
