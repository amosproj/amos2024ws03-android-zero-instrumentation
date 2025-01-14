// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

/**
 * Convert [Configuration] to UI Options ([BackendFeatureOptions] ). Show as enabled depending on
 * the PIDs the screen is configuring.
 */
fun Configuration.toUIOptionsForPids(
    relevantPids: List<UInt>? = null
): List<BackendFeatureOptions> {
    val options = mutableListOf<BackendFeatureOptions>()
    if (relevantPids != null) {
        options.add(
            this.vfsWrite?.let {
                BackendFeatureOptions.VfsWriteOption(
                    enabled = it.entries.keys.anyPidsEnabled(relevantPids),
                    pids = it.entries.keys,
                )
            } ?: BackendFeatureOptions.VfsWriteOption(enabled = false, pids = setOf())
        )

        options.add(
            this.sysSendmsg?.let {
                BackendFeatureOptions.SendMessageOption(
                    enabled = it.entries.keys.anyPidsEnabled(relevantPids),
                    pids = it.entries.keys,
                )
            } ?: BackendFeatureOptions.SendMessageOption(enabled = false, pids = setOf())
        )

        options.add(
            this.jniReferences?.let {
                BackendFeatureOptions.JniReferencesOption(
                    enabled = it.pids.anyPidsEnabled(relevantPids),
                    pids = it.pids.toSet(),
                )
            } ?: BackendFeatureOptions.JniReferencesOption(enabled = false, pids = setOf())
        )

        options.add(
            this.sysSigquit?.let {
                BackendFeatureOptions.SigquitOption(
                    enabled = it.pids.anyPidsEnabled(relevantPids),
                    pids = it.pids.toSet(),
                )
            } ?: BackendFeatureOptions.SigquitOption(enabled = false, pids = setOf())
        )

        this.uprobes
            .filter { it.pid == null || relevantPids.contains(it.pid!!.toUInt()) }
            .forEach { uprobeConfig ->
                options.add(
                    BackendFeatureOptions.UprobeOption(
                        enabled = true, // uprobe options are either active or not visible
                        method = uprobeConfig.fnName,
                        pids = uprobeConfig.pid?.let { setOf(it) } ?: setOf(),
                        odexFilePath = uprobeConfig.target,
                        offset = uprobeConfig.offset,
                    )
                )
            }
    }
    return options.toList()
}
