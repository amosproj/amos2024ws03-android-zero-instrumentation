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
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import timber.log.Timber

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
            .filter { it is Event.VfsWrite }
            .map {
                if (it is Event.VfsWrite) {
                    WriteEvent.VfsWriteEvent(it.fp, it.pid, it.bytesWritten, it.beginTimeStamp)
                } else throw Exception("only for the compiler")
            }

    override suspend fun sendMessageEvents(pids: List<UInt>): Flow<WriteEvent.SendMessageEvent> =
        clientFactory
            .connect()
            .initStream()
            .filter { it is Event.SysSendmsg }
            .map {
                if (it is Event.SysSendmsg) {
                    WriteEvent.SendMessageEvent(
                        it.fd.toULong(),
                        it.pid,
                        it.tid,
                        it.beginTimeStamp,
                        it.durationMicroSec,
                    )
                } else throw Exception("only for the compiler")
            }
}
