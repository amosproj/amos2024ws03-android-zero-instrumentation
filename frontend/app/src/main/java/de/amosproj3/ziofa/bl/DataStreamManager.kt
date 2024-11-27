// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl

import de.amosproj3.ziofa.api.DataStreamProvider
import de.amosproj3.ziofa.api.WriteEvent
import de.amosproj3.ziofa.client.ClientFactory
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import timber.log.Timber
import uniffi.shared.EventData

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
            .map { it.eventData }
            .filter { it is EventData.VfsWrite }
            .map {
                if (it is EventData.VfsWrite) {
                    WriteEvent.VfsWriteEvent(
                        it.v1.fp,
                        it.v1.pid,
                        it.v1.bytesWritten,
                        it.v1.beginTimeStamp,
                    )
                } else throw Exception("only for the compiler")
            }

    override suspend fun sendMessageEvents(pids: List<UInt>): Flow<WriteEvent.SendMessageEvent> =
        clientFactory
            .connect()
            .initStream()
            .map { it.eventData }
            .filter { it is EventData.SysSendmsg }
            .map {
                if (it is EventData.SysSendmsg) {
                    WriteEvent.SendMessageEvent(
                        it.v1.fd.toULong(),
                        it.v1.pid,
                        it.v1.tid,
                        it.v1.beginTimeStamp,
                        it.v1.durationMicroSec,
                    )
                } else throw Exception("only for the compiler")
            }
}
