// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlin.random.Random
import kotlin.random.nextUInt
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

const val alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

object RustClient : Client {
    private var configuration: Configuration =
        Configuration(
            vfsWrite = VfsWriteConfig(mapOf(1234u to 30000u, 43124u to 20000u)),
            sysSendmsg = SysSendmsgConfig(mapOf(1234u to 30000u, 43124u to 20000u)),
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

    private val processes =
        alphabet.indices.map {
            Process(
                pid = Random.nextUInt(1000u).toInt(),
                ppid = Random.nextUInt(1000u).toInt(),
                state = "R",
                cmd = Command.Comm("/bin/sh/${alphabet.substring(it, it + 1)}"),
            )
        }

    override suspend fun listProcesses(): List<Process> {
        return processes
    }

    override suspend fun getConfiguration(): Configuration {
        return configuration
    }

    override suspend fun setConfiguration(configuration: Configuration): UInt {
        this.configuration = configuration

        return 0u
    }

    override suspend fun initStream(): Flow<Event> = flow {
        while (true) {
            delay(Random.nextUInt(500u).toLong())

            configuration.vfsWrite?.entries?.keys?.forEach {
                emit(
                    Event.VfsWrite(
                        pid = it,
                        tid = 1234u,
                        fp = 125123123u,
                        bytesWritten = 123121u,
                        beginTimeStamp = 12312412u,
                    )
                )
            }
            configuration.sysSendmsg?.entries?.keys?.forEach {
                emit(
                    Event.SysSendmsg(
                        pid = it,
                        tid = 1234u,
                        fd = 125123123u,
                        durationNanoSecs =
                            (System.currentTimeMillis() + Random.nextLong(1000)).toULong(),
                        beginTimeStamp = System.currentTimeMillis().toULong(),
                    )
                )
            }
        }
    }

    override suspend fun getOdexFiles(pid: UInt): Flow<StringResponse> = flow {
        emit(StringResponse(name = "/system/framework/oat/x86_64/android.test.base.odex"))
        emit(StringResponse(name = "/system/framework/oat/x86_64/android.hidl.base-V1.0-java.odex"))
        emit(StringResponse(name = "/system/framework/oat/x86_64/org.apache.http.legacy.odex"))
        emit(
            StringResponse(
                name = "/system/framework/oat/x86_64/android.hidl.manager-V1.0-java.odex"
            )
        )
        emit(StringResponse(name = "/system_ext/framework/oat/x86_64/androidx.window.sidecar.odex"))
        emit(
            StringResponse(
                name =
                    "/data/app/~~0cD8TtY5ggbzXOrlKANgwQ==/de.amosproj3.ziofa-Sm8ZemAtgxCr5VAK1Cwi8Q==/oat/x86_64/base.odex"
            )
        )
        emit(
            StringResponse(
                name = "/system_ext/framework/oat/x86_64/androidx.window.extensions.odex"
            )
        )
    }

    override suspend fun getSymbols(odexFilePath: String): Flow<Symbol> = flow {
        emit(
            Symbol(
                method =
                    "void androidx.compose.material3.SearchBarDefaults\$InputField\$1\$1.<init>(kotlin.jvm.functions.Function1)",
                offset = 6012800u,
            )
        )
        emit(
            Symbol(
                method =
                    "void kotlin.collections.ArraysKt___ArraysKt\$asSequence\$\$inlined\$Sequence\$2.<init>(byte[])",
                offset = 5915712u,
            )
        )
        emit(
            Symbol(
                method =
                    "boolean androidx.compose.ui.platform.ViewLayer\$Companion.getHasRetrievedMethod()",
                offset = 24010112u,
            )
        )
        emit(
            Symbol(
                method =
                    "androidx.core.app.NotificationCompat\$BubbleMetadata androidx.core.app.NotificationCompat\$BubbleMetadata\$Api29Impl.fromPlatform(android.app.Notification\$BubbleMetadata)",
                offset = 25453376u,
            )
        )
        emit(
            Symbol(
                method = "byte androidx.emoji2.text.flatbuffer.FlexBuffers\$Blob.get(int)",
                offset = 26906336u,
            )
        )
    }
}

class RustClientFactory(val url: String) : ClientFactory {
    override suspend fun connect(): Client {
        return RustClient
    }
}
