// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.configuration

import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.SysSigquitConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.shared.DURATION_THRESHOLD

fun Configuration.applyChange(
    action: ConfigurationAction.ChangeFeature,
): Configuration {

    val feature = action.backendFeature
    val enable = action.enable
    val pids = action.pids

    return when (feature) {
        is BackendFeatureOptions.VfsWriteOption ->
            this.copy(
                vfsWrite = this.vfsWrite?.updatePIDs(
                    pidsToAdd = if (enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf(),
                    pidsToRemove = if (!enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf()
                )
            )

        is BackendFeatureOptions.SendMessageOption ->
            this.copy(
                sysSendmsg = this.sysSendmsg?.updatePIDs(
                    pidsToAdd = if (enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf(),
                    pidsToRemove = if (!enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf()
                )
            )

        is BackendFeatureOptions.JniReferencesOption ->
            this.copy(
                jniReferences = this.jniReferences?.updatePIDs(
                    pidsToAdd = if (enable) pids else setOf(),
                    pidsToRemove = if (!enable) pids else setOf()
                )
            )

        is BackendFeatureOptions.UprobeOption -> {
            val uprobeUpdate = pids.map {
                UprobeConfig(
                    fnName = feature.method,
                    target = feature.odexFilePath,
                    offset = feature.offset,
                    pid = it,
                )
            }
            this.copy(
                uprobes = this.uprobes.updateUProbes(
                    pidsToAdd = if (enable) uprobeUpdate else listOf(),
                    pidsToRemove = if (!enable) uprobeUpdate else listOf()
                )
            )
        }

    }
}

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
    pidsToAdd: Set<UInt> = setOf(),
    pidsToRemove: Set<UInt> = setOf(),
): JniReferencesConfig {
    val config = this ?: JniReferencesConfig(listOf())
    return config.copy(pids = config.pids.plus(pidsToAdd).minus(pidsToRemove.toSet()))
}

fun SysSigquitConfig?.updatePIDs(
    pidsToAdd: List<UInt> = listOf(),
    pidsToRemove: List<UInt> = listOf(),
): SysSigquitConfig {
    val config = this ?: SysSigquitConfig(listOf())
    return config.copy(pids = config.pids.plus(pidsToAdd).minus(pidsToRemove.toSet()))
}
