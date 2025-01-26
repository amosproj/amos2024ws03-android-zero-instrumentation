// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import android.os.SystemClock
import kotlin.random.Random
import kotlin.random.nextUInt
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.merge
import kotlin.random.nextULong

var gcPids = setOf<UInt>()

object RustClient : Client {
    private var configuration: Configuration =
        Configuration(
            vfsWrite = null,
            sysSendmsg = null,
            uprobes = listOf(),
            jniReferences = null,
            sysSigquit = null,
            gc = null,
            sysFdTracking = null
        )

    override suspend fun serverCount(): Flow<UInt> = flowOf()

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

    override suspend fun indexSymbols() {
        // NOP
        delay(2000)
    }

    private val processes =
        processesList

    override suspend fun listProcesses(): List<Process> {
        return processes
    }

    override suspend fun getConfiguration(): Configuration {
        return configuration
    }

    override suspend fun setConfiguration(configuration: Configuration) {
        this.configuration = configuration
    }

    override suspend fun initStream(): Flow<Event> = merge(
        vfsWriteMockEvents(500),
        sendMsgMockEvents(500),
        jniReferencesMockEvents(700),
        sysSigQuitMockEvents(3000),
        sysFdTrackingMockEvents(1000)
    )

    private fun vfsWriteMockEvents(emissionDelayBoundMillis: Int) = flow {
        while (true) {
            configuration.vfsWrite?.entries?.keys?.forEach {
                emit(
                    Event.VfsWrite(
                        pid = it,
                        tid = it + 1u,
                        fp = listOf(0uL, 1uL, 2uL, 3uL).random(),
                        bytesWritten = listOf(8uL, 16uL, 32uL, 64uL).random(),
                        beginTimeStamp = SystemClock.elapsedRealtimeNanos().toULong()
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }

    private fun sendMsgMockEvents(emissionDelayBoundMillis: Int) = flow {

        while (true) {
            configuration.sysSendmsg?.entries?.keys?.forEach {
                emit(
                    Event.SysSendmsg(
                        pid = it,
                        tid = it + 1u,
                        fd = listOf(0uL, 1uL, 2uL, 3uL).random(),
                        durationNanoSecs = 1000000u + Random.nextULong(4000000u),
                        beginTimeStamp = SystemClock.elapsedRealtimeNanos().toULong(),
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }

    private fun jniReferencesMockEvents(emissionDelayBoundMillis: Int) = flow {
        while (true) {
            configuration.jniReferences?.pids?.forEach {
                val rnd = Random.nextFloat()
                val jniMethodName =
                    if (rnd > 0.33f) Event.JniReferences.JniMethodName.AddGlobalRef
                    else Event.JniReferences.JniMethodName.DeleteGlobalRef
                emit(
                    Event.JniReferences(
                        pid = it,
                        tid = it + 1u,
                        beginTimeStamp = SystemClock.elapsedRealtimeNanos().toULong(),
                        jniMethodName = jniMethodName
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }

    private fun sysSigQuitMockEvents(emissionDelayBoundMillis: Int) = flow {
        while (true) {
            configuration.sysSigquit?.pids?.forEach {
                emit(
                    Event.SysSigquit(
                        pid = it,
                        tid = it + 1u,
                        timeStamp = SystemClock.elapsedRealtimeNanos().toULong(),
                        targetPid = it.toULong(),
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }

    private fun sysFdTrackingMockEvents(emissionDelayBoundMillis: Int) = flow {
        while (true) {
            configuration.sysFdTracking?.pids?.forEach {
                val rnd = Random.nextFloat()
                val syFdMethod =
                    if (rnd > 0.33f) Event.SysFdTracking.SysFdAction.Created
                    else Event.SysFdTracking.SysFdAction.Destroyed
                emit(
                    Event.SysFdTracking(
                        pid = it,
                        tid = it + 1u,
                        timeStamp = SystemClock.elapsedRealtimeNanos().toULong(),
                        fdAction = syFdMethod,
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }


    override suspend fun getOdexFiles(pid: UInt): Flow<String> = mockOdexFileFlow

    override suspend fun getSoFiles(pid: UInt): Flow<String> = mockSoFileFlow

    override suspend fun getSymbols(filePath: String): Flow<Symbol> = mockSymbolFlow
}

class RustClientFactory(val url: String) : ClientFactory {
    override suspend fun connect(): Client {
        return RustClient
    }
}
