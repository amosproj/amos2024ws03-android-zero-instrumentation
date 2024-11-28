// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl

import de.amosproj3.ziofa.api.DataStreamProvider
import de.amosproj3.ziofa.api.WriteEvent
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Event
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import timber.log.Timber

// TODO: use a single sharedFlow and then different filters on top of that
// otherwise we are sending all the data multiple times from server to client
class DataStreamManager(private val clientFactory: ClientFactory) : DataStreamProvider {

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

    override suspend fun vfsWriteEvents(pids: List<UInt>): Flow<WriteEvent.VfsWriteEvent> =
        clientFactory
            .connect()
            .initStream()
            .mapNotNull { it as? Event.VfsWrite }
            .map {
                WriteEvent.VfsWriteEvent(
                    fd = it.fp,
                    pid = it.pid,
                    size = it.bytesWritten,
                    timestampMillis = it.beginTimeStamp,
                )
            }

    override suspend fun sendMessageEvents(pids: List<UInt>): Flow<WriteEvent.SendMessageEvent> =
        clientFactory
            .connect()
            .initStream()
            .mapNotNull { it as? Event.SysSendmsg }
            .map {
                WriteEvent.SendMessageEvent(
                    fd = it.fd,
                    pid = it.pid,
                    tid = it.tid,
                    beginTimestamp = it.beginTimeStamp,
                    durationNanos = it.durationNanoSecs,
                )
            }
}
