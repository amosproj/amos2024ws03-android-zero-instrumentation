// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.client.GcConfig
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysFdTrackingConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.SysSigquitConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.persistentListOf
import kotlinx.collections.immutable.toImmutableList

/**
 * Convert [Configuration] to UI Options ([BackendFeatureOptions] ). Show as enabled depending on
 * the PIDs the screen is configuring.
 */
fun Configuration.toUIOptionsForPids(
    relevantPids: List<UInt>? = null
): ImmutableList<BackendFeatureOptions> {
    val backendConfig = this
    if (relevantPids == null) {
        return persistentListOf()
    }
    return buildList {
            add(backendConfig.vfsWrite.toBackendFeatureOption(relevantPids))
            add(backendConfig.sysSendmsg.toBackendFeatureOption(relevantPids))
            add(backendConfig.jniReferences.toBackendFeatureOption(relevantPids))
            add(backendConfig.sysSigquit.toBackendFeatureOption(relevantPids))
            add(backendConfig.gc.toBackendFeatureOption(relevantPids))
            add(backendConfig.sysFdTracking.toBackendFeatureOption(relevantPids))
            addAll(
                backendConfig.uprobes
                    .filter { it.pid == null || relevantPids.contains(it.pid!!.toUInt()) }
                    .map { uprobeConfig ->
                        BackendFeatureOptions.UprobeOption(
                            enabled = true, // uprobe options are either active or not visible
                            method = uprobeConfig.fnName,
                            pids = uprobeConfig.pid?.let { setOf(it) } ?: setOf(),
                            odexFilePath = uprobeConfig.target,
                            offset = uprobeConfig.offset,
                        )
                    }
            )
        }
        .toImmutableList()
}

fun VfsWriteConfig?.toBackendFeatureOption(relevantPids: List<UInt>?) =
    this?.let {
        BackendFeatureOptions.VfsWriteOption(
            enabled = it.pids.anyPidsEnabled(relevantPids),
            pids = it.pids.toSet(),
        )
    } ?: BackendFeatureOptions.VfsWriteOption(enabled = false, pids = setOf())

fun SysSendmsgConfig?.toBackendFeatureOption(relevantPids: List<UInt>?) =
    this?.let {
        BackendFeatureOptions.SendMessageOption(
            enabled = it.entries.keys.anyPidsEnabled(relevantPids),
            pids = it.entries.keys,
        )
    } ?: BackendFeatureOptions.SendMessageOption(enabled = false, pids = setOf())

fun JniReferencesConfig?.toBackendFeatureOption(relevantPids: List<UInt>?) =
    this?.let {
        BackendFeatureOptions.JniReferencesOption(
            enabled = it.pids.anyPidsEnabled(relevantPids),
            pids = it.pids.toSet(),
        )
    } ?: BackendFeatureOptions.JniReferencesOption(enabled = false, pids = setOf())

fun SysSigquitConfig?.toBackendFeatureOption(relevantPids: List<UInt>?) =
    this?.let {
        BackendFeatureOptions.SigquitOption(
            enabled = it.pids.anyPidsEnabled(relevantPids),
            pids = it.pids.toSet(),
        )
    } ?: BackendFeatureOptions.SigquitOption(enabled = false, pids = setOf())

fun GcConfig?.toBackendFeatureOption(relevantPids: List<UInt>?) =
    this?.let {
        BackendFeatureOptions.GcOption(
            enabled = it.pids.anyPidsEnabled(relevantPids),
            pids = it.pids,
        )
    } ?: BackendFeatureOptions.GcOption(enabled = false, setOf())

fun SysFdTrackingConfig?.toBackendFeatureOption(relevantPids: List<UInt>?) =
    this?.let {
        BackendFeatureOptions.OpenFileDescriptors(
            enabled = it.pids.anyPidsEnabled(relevantPids),
            pids = it.pids.toSet(),
        )
    } ?: BackendFeatureOptions.OpenFileDescriptors(enabled = false, setOf())
