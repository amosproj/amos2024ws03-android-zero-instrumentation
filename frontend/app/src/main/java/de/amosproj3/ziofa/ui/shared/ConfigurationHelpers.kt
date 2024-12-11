// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.api.configuration.ConfigurationUpdate
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

fun VfsWriteConfig?.updatePIDs(
    pidsToAdd: Set<Map.Entry<UInt, ULong>> = setOf(),
    pidsToRemove: Set<Map.Entry<UInt, ULong>> = setOf(),
): VfsWriteConfig {
    val config = this ?: VfsWriteConfig(mapOf())
    return config.copy(
        entries =
        config.entries.entries.plus(pidsToAdd).minus(pidsToRemove).associate {
            it.key to it.value
        }
    )
}

fun SysSendmsgConfig?.updatePIDs(
    pidsToAdd: Set<Map.Entry<UInt, ULong>> = setOf(),
    pidsToRemove: Set<Map.Entry<UInt, ULong>> = setOf(),
): SysSendmsgConfig {
    val config = this ?: SysSendmsgConfig(mapOf())
    return config.copy(
        entries =
        config.entries.entries.plus(pidsToAdd).minus(pidsToRemove).associate {
            it.key to it.value
        }
    )
}

fun List<UprobeConfig>?.updateUProbes(
    pidsToAdd: List<UprobeConfig> = listOf(),
    pidsToRemove: List<UprobeConfig> = listOf(),
): List<UprobeConfig> {
    val config = this ?: listOf()
    return config.minus(pidsToRemove.toSet()).plus(pidsToAdd).toSet().toList()
}

fun JniReferencesConfig?.updatePIDs(
    pidsToAdd: List<UInt> = listOf(),
    pidsToRemove: List<UInt> = listOf(),
): JniReferencesConfig {
    val config = this ?: JniReferencesConfig(listOf())
    return config.copy(pids = config.pids.plus(pidsToAdd).minus(pidsToRemove.toSet()))
}

/** Show as enabled depending on the PIDs the screen is configuring. */
fun ConfigurationUpdate.Valid.toUIOptionsForPids(
    relevantPids: List<UInt>? = null
): List<BackendFeatureOptions> {
    val options = mutableListOf<BackendFeatureOptions>()
    if (relevantPids != null) {
        options.add(
            this.configuration.vfsWrite?.let {
                BackendFeatureOptions.VfsWriteOption(
                    enabled = it.entries.keys.anyPidsEnabled(relevantPids),
                    pids = it.entries.keys,
                )
            } ?: BackendFeatureOptions.VfsWriteOption(enabled = false, pids = setOf())
        )

        options.add(
            this.configuration.sysSendmsg?.let {
                BackendFeatureOptions.SendMessageOption(
                    enabled = it.entries.keys.anyPidsEnabled(relevantPids),
                    pids = it.entries.keys,
                )
            } ?: BackendFeatureOptions.SendMessageOption(enabled = false, pids = setOf())
        )

        options.add(
            this.configuration.jniReferences?.let {
                BackendFeatureOptions.JniReferencesOption(
                    enabled = it.pids.anyPidsEnabled(relevantPids),
                    pids = it.pids.toSet()
                )
            } ?: BackendFeatureOptions.JniReferencesOption(enabled = false, pids = setOf())
        )

        this.configuration.uprobes
            .filter { it.pid == null || relevantPids.contains(it.pid!!.toUInt()) }
            .forEach { uprobeConfig ->
                options.add(
                    BackendFeatureOptions.UprobeOption(
                        enabled = true, // uprobe options are either active or not visible
                        method = uprobeConfig.fnName,
                        pids = uprobeConfig.pid?.let { setOf(it.toUInt()) } ?: setOf(),
                        odexFilePath = uprobeConfig.target,
                        offset = uprobeConfig.offset,
                    )
                )
            }
    }

    return options.toList()
}
