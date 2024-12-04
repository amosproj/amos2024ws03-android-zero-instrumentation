// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl

import de.amosproj3.ziofa.api.BackendEvent
import de.amosproj3.ziofa.api.DataStreamProvider
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Event
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.shareIn
import timber.log.Timber

class DataStreamManager(private val clientFactory: ClientFactory, coroutineScope: CoroutineScope) :
    DataStreamProvider {

    private val dataFlow =
        flow { clientFactory.connect().initStream().collect { emit(it) } }
            .shareIn(coroutineScope, SharingStarted.Lazily)

    override suspend fun counter(ebpfProgramName: String): Flow<UInt> {
        return clientFactory
            .connect()
            .also {
                try {
                    it.load()
                    // default wifi interface on android, now configurable
                    it.attach("wlan0")
                    it.startCollecting()
                } catch (e: Exception) {
                    Timber.e(e.stackTraceToString())
                }
            }
            .serverCount()
    }

    override suspend fun vfsWriteEvents(pids: List<UInt>?): Flow<BackendEvent.VfsWriteEvent> =
        dataFlow
            .mapNotNull { it as? Event.VfsWrite }
            .filter { it.pid.isGlobalRequestedOrPidConfigured(pids) }
            .map {
                BackendEvent.VfsWriteEvent(
                    fd = it.fp,
                    pid = it.pid,
                    size = it.bytesWritten,
                    timestampMillis = it.beginTimeStamp,
                )
            }

    override suspend fun sendMessageEvents(pids: List<UInt>?): Flow<BackendEvent.SendMessageEvent> =
        dataFlow
            .mapNotNull { it as? Event.SysSendmsg }
            .filter { it.pid.isGlobalRequestedOrPidConfigured(pids) }
            .map {
                BackendEvent.SendMessageEvent(
                    fd = it.fd,
                    pid = it.pid,
                    tid = it.tid,
                    beginTimestamp = it.beginTimeStamp,
                    durationNanos = it.durationNanoSecs,
                )
            }

    private fun UInt.isGlobalRequestedOrPidConfigured(pids: List<UInt>?) =
        pids?.contains(this) ?: true
}
