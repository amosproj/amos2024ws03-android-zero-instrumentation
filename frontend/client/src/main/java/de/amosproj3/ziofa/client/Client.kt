package de.amosproj3.ziofa.client

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.shareIn

interface Client {
    val serverCount: SharedFlow<UInt>

    suspend fun loadProgram(name: String)
}

interface ClientFactory {
    suspend fun connect(scope: CoroutineScope, url: String): Client
}


class RustClient(
    scope: CoroutineScope,
    private val inner: uniffi.client.Client
) : Client {

    override val serverCount = flow {
        val stream = inner.serverCount()

        while (true) {
            stream.next()?.also {
                emit(it)
            } ?: break;
        }
    }.shareIn(scope, SharingStarted.Lazily)

    override suspend fun loadProgram(name: String) {
        inner.loadProgram(name)
    }
}

class RustClientFactory : ClientFactory {
    override suspend fun connect(scope: CoroutineScope, url: String): Client {
        return RustClient(scope, uniffi.client.Client.connect(url))
    }
}