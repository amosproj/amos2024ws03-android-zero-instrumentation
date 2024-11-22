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
import uniffi.shared.EbpfEntry
import uniffi.shared.Process
import uniffi.shared.UprobeConfig

const val alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

object RustClient : Client {
    private var configuration: Configuration =
        Configuration(
            listOf(
                EbpfEntry(
                    "Test HR name",
                    "this is a test",
                    "ebpf_name",
                    12345u,
                    UprobeConfig(0u, "target", 54321),
                    "hook",
                    false,
                )
            )
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
}

class RustClientFactory(val url: String) : ClientFactory {
    override suspend fun connect(): Client {
        return RustClient
    }
}
