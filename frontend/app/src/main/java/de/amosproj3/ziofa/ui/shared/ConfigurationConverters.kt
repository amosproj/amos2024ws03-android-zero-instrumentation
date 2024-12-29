// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

/**
 * Convert ConfigurationUpdate to UI Options ([BackendFeatureOptions] ). Show as enabled depending
 * on the PIDs the screen is configuring.
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

/**
 * Convert [BackendFeatureOptions] from UI to configuration and set the changes in the local
 * configuration.
 */
fun BackendFeatureOptions.toConfigurationChangeAction(
    pids: Set<UInt>,
    active: Boolean,
): ConfigurationAction.Change =
    when (this) {
        is BackendFeatureOptions.VfsWriteOption ->
            ConfigurationAction.Change(
                enable = active,
                vfsWriteFeature = VfsWriteConfig(pids.associateWith { DURATION_THRESHOLD }),
            )

        is BackendFeatureOptions.SendMessageOption ->
            ConfigurationAction.Change(
                enable = active,
                sendMessageFeature =
                SysSendmsgConfig(
                    pids.associateWith { DURATION_THRESHOLD }
                    // TODO this is not a duration
                ),
            )

        is BackendFeatureOptions.UprobeOption ->
            ConfigurationAction.Change(
                enable = active,
                uprobesFeature =
                pids.map {
                    UprobeConfig(
                        fnName = this.method,
                        target = this.odexFilePath,
                        offset = this.offset,
                        pid = it,
                    )
                },
            )

        is BackendFeatureOptions.JniReferencesOption ->
            ConfigurationAction.Change(
                enable = active,
                jniReferencesFeature = JniReferencesConfig(pids.toList()),
            )

        else -> throw NotImplementedError("NO SUPPORT YET")
    }
