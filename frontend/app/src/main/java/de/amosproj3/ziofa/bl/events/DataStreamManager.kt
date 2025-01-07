// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.events

import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Event
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.shareIn

class DataStreamManager(private val clientFactory: ClientFactory, coroutineScope: CoroutineScope) :
    DataStreamProvider {

    private val dataFlow =
        flow { clientFactory.connect().initStream().collect { emit(it) } }
            .shareIn(coroutineScope, SharingStarted.Lazily)

    override fun vfsWriteEvents(pids: List<UInt>?): Flow<Event.VfsWrite> =
        dataFlow
            .mapNotNull { it as? Event.VfsWrite }
            .filter { it.pid.isGlobalRequestedOrPidConfigured(pids) }

    override fun sendMessageEvents(pids: List<UInt>?): Flow<Event.SysSendmsg> =
        dataFlow
            .mapNotNull { it as? Event.SysSendmsg }
            .filter { it.pid.isGlobalRequestedOrPidConfigured(pids) }

    override fun jniReferenceEvents(pids: List<UInt>?): Flow<Event.JniReferences> =
        dataFlow
            .mapNotNull { it as? Event.JniReferences }
            .filter { it.pid.isGlobalRequestedOrPidConfigured(pids) }

    override fun sigquitEvents(pids: List<UInt>?): Flow<Event.SysSigquit> =
        dataFlow
            .mapNotNull { it as? Event.SysSigquit }
            .filter { it.pid.isGlobalRequestedOrPidConfigured(pids) }

    private fun UInt.isGlobalRequestedOrPidConfigured(pids: List<UInt>?) =
        pids?.contains(this) ?: true
}
