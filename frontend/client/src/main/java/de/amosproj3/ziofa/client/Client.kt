// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn

interface Client {
    val serverCount: StateFlow<UInt>

    suspend fun load()

    suspend fun attach(iface: String)

    suspend fun unload()

    suspend fun detach(iface: String)

    suspend fun startCollecting()

    suspend fun stopCollecting()
}

interface ClientFactory {
    suspend fun connect(scope: CoroutineScope, url: String): Client
}

class RustClient(scope: CoroutineScope, private val inner: uniffi.client.Client) : Client {

    override val serverCount =
        flow {
                val stream = inner.serverCount()
                Log.e("client", "init")
                while (true) {
                    stream.next()?.also { emit(it) } ?: break
                }
            }
            .stateIn(scope, SharingStarted.Lazily, 0u)

    override suspend fun load() = inner.load()

    override suspend fun attach(iface: String) = inner.attach(iface)

    override suspend fun unload() = inner.unload()

    override suspend fun detach(iface: String) = inner.detach(iface)

    override suspend fun startCollecting() = inner.startCollecting()

    override suspend fun stopCollecting() = inner.stopCollecting()
}

class RustClientFactory : ClientFactory {
    override suspend fun connect(scope: CoroutineScope, url: String): Client {
        return RustClient(scope, uniffi.client.Client.connect(url))
    }
}
