// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlin.random.Random
import kotlin.random.nextUInt
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import uniffi.shared.Cmd
import uniffi.shared.Configuration
import uniffi.shared.Event
import uniffi.shared.EventData
import uniffi.shared.Process
import uniffi.shared.SysSendmsgConfig
import uniffi.shared.SysSendmsgEvent
import uniffi.shared.VfsWriteConfig
import uniffi.shared.VfsWriteEvent

const val alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

object RustClient : Client {
    private var configuration: Configuration =
        Configuration(
            vfsWrite = VfsWriteConfig(listOf(1234u, 43124u)),
            sysSendmsg = SysSendmsgConfig(listOf(1234u, 43124u)),
            uprobes = listOf(),
        )

    override suspend fun serverCount(): Flow<UInt> = flow {
        var ctr = 0u
        while (true) {
            delay(Random.nextUInt(500u).toLong())
            ctr++
            emit(ctr)
        }
    }

    override suspend fun load() {
        // NOP
    }

    override suspend fun attach(iface: String) {
        // NOP
    }

    override suspend fun unload() {
        // NOP
    }

    override suspend fun detach(iface: String) {
        // NOP
    }

    override suspend fun startCollecting() {
        // NOP
    }

    override suspend fun stopCollecting() {
        // NOP
    }

    override suspend fun checkServer() {
        // NOP
    }

    override suspend fun listProcesses(): List<Process> {
        return alphabet.indices.map {
            Process(
                pid = Random.nextUInt(1000u).toInt(),
                ppid = Random.nextUInt(1000u).toInt(),
                state = "R",
                cmd = Cmd.Comm("/bin/sh/${alphabet.substring(it, it + 1)}"),
            )
        }
    }

    override suspend fun getConfiguration(): Configuration {
        return configuration
    }

    override suspend fun setConfiguration(configuration: Configuration) {
        this.configuration = configuration
    }

    override suspend fun initStream(): Flow<Event> = flow {
        while (true) {
            delay(Random.nextUInt(500u).toLong())
            emit(
                Event(
                    EventData.VfsWrite(
                        VfsWriteEvent(
                            pid = 12415u,
                            tid = 1234u,
                            fp = 125123123u,
                            bytesWritten = 123121u,
                            beginTimeStamp = 12312412u,
                        )
                    )
                )
            )

            emit(
                Event(
                    EventData.SysSendmsg(
                        SysSendmsgEvent(
                            pid = 12345u,
                            tid = 1234u,
                            fd = 125123123,
                            durationMicroSec =
                                (System.currentTimeMillis() + Random.nextLong(1000)).toULong(),
                            beginTimeStamp = System.currentTimeMillis().toULong(),
                        )
                    )
                )
            )
        }
    }
}

class RustClientFactory(val url: String) : ClientFactory {
    override suspend fun connect(): Client {
        return RustClient
    }
}
