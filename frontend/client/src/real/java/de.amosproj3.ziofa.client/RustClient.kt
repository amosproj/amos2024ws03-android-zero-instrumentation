// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import uniffi.shared.Configuration
import uniffi.shared.Event
import uniffi.shared.Process

class RustClient(private val inner: uniffi.client.Client) : Client {

    override suspend fun serverCount(): Flow<UInt> = inner.serverCountFlow()

    override suspend fun load() = inner.load()

    override suspend fun attach(iface: String) = inner.attach(iface)

    override suspend fun unload() = inner.unload()

    override suspend fun detach(iface: String) = inner.detach(iface)

    override suspend fun startCollecting() = inner.startCollecting()

    override suspend fun stopCollecting() = inner.stopCollecting()

    override suspend fun checkServer() = inner.checkServer()

    override suspend fun listProcesses(): List<Process> = inner.listProcesses()

    override suspend fun getConfiguration(): Configuration = inner.getConfiguration()

    override suspend fun setConfiguration(configuration: Configuration) =
        inner.setConfiguration(configuration)

    override suspend fun initStream(): Flow<Event> = inner.initStreamFlow()
}

class RustClientFactory(val url: String) : ClientFactory {

    private val _m = Mutex()
    private var client: RustClient? = null

    override suspend fun connect(): Client {
        client?.also {
            return it
        }

        _m.withLock {
            client?.also {
                return it
            }
            val c = RustClient(uniffi.client.Client.connect(url))
            client = c
            return c
        }
    }
}

fun uniffi.client.Client.serverCountFlow() = flow {
    serverCount().use { stream ->
        while (true) {
            stream.next()?.also { count -> emit(count) } ?: break
        }
    }
}

fun uniffi.client.Client.initStreamFlow() = flow {
    initStream().use { stream ->
        while (true) {
            stream.next()?.also { event -> emit(event) } ?: break
        }
    }
}
