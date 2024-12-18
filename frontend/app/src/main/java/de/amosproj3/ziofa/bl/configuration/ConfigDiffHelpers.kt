// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.configuration

import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig

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
