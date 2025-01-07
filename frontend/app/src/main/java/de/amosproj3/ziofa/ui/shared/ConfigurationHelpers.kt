// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.bl.configuration.updatePIDs
import de.amosproj3.ziofa.bl.configuration.updateUProbes
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.SysSigquitConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig

fun Configuration.merge(vfsWriteConfig: VfsWriteConfig?, enable: Boolean) =
    vfsWriteConfig?.let { requestedChanges ->
        this.vfsWrite.updatePIDs(
            pidsToAdd = if (enable) requestedChanges.entries.entries else setOf(),
            pidsToRemove = if (!enable) requestedChanges.entries.entries else setOf(),
        )
    } ?: this.vfsWrite

fun Configuration.merge(sysSendmsgConfig: SysSendmsgConfig?, enable: Boolean) =
    sysSendmsgConfig?.let { requestedChanges ->
        this.sysSendmsg.updatePIDs(
            pidsToAdd = if (enable) requestedChanges.entries.entries else setOf(),
            pidsToRemove = if (!enable) requestedChanges.entries.entries else setOf(),
        )
    } ?: this.sysSendmsg

fun Configuration.merge(uprobeConfigs: List<UprobeConfig>?, enable: Boolean) =
    uprobeConfigs?.let { requestedChanges ->
        this.uprobes.updateUProbes(
            pidsToAdd = if (enable) requestedChanges else listOf(),
            pidsToRemove = if (!enable) requestedChanges else listOf(),
        )
    } ?: this.uprobes

fun Configuration.merge(jniReferencesConfig: JniReferencesConfig?, enable: Boolean) =
    jniReferencesConfig?.let { requestedChanges ->
        this.jniReferences.updatePIDs(
            pidsToAdd = if (enable) requestedChanges.pids else listOf(),
            pidsToRemove = if (!enable) requestedChanges.pids else listOf(),
        )
    } ?: this.jniReferences

fun Configuration.merge(sysSigquitConfig: SysSigquitConfig?, enable: Boolean) =
    sysSigquitConfig?.let { requestedChanges ->
        this.sysSigquit.updatePIDs(
            pidsToAdd = if (enable) requestedChanges.pids else listOf(),
            pidsToRemove = if (!enable) requestedChanges.pids else listOf(),
        )
    } ?: this.sysSigquit
