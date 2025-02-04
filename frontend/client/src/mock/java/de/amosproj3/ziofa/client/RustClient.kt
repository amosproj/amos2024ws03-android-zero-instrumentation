// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import android.os.SystemClock
import kotlin.random.Random
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.merge
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import kotlin.time.Duration.Companion.nanoseconds

object RustClient : Client {
    private var configuration: Configuration =
        Configuration(
            vfsWrite = null,
            sysSendmsg = null,
            uprobes = listOf(),
            jniReferences = null,
            sysSigquit = null,
            gc = null,
            sysFdTracking = null,
        )

    override suspend fun indexSymbols() {
        // NOP
        delay(2000)
    }

    private val processes = processesList

    override suspend fun listProcesses(): List<Process> {
        return processes
    }

    override suspend fun getConfiguration(): Configuration {
        return configuration
    }

    override suspend fun setConfiguration(configuration: Configuration) {
        this.configuration = configuration
    }

    override suspend fun initStream(): Flow<Event> =
        merge(
            vfsWriteMockEvents(1000),
            sendMsgMockEvents(500),
            jniReferencesMockEvents(700),
            sysSigQuitMockEvents(5000),
            sysFdTrackingMockEvents(1000),
            gcMockEvents(4000),
        )

    private fun vfsWriteMockEvents(emissionDelayBoundMillis: Int) = flow {
        while (true) {
            configuration.vfsWrite?.pids?.forEach {
                emit(
                    Event.VfsWrite(
                        pid = it,
                        tid = it + 1u,
                        fp =
                        listOf(
                            "/data/vendor/navigationd/backup.sqlite",
                            "/data/vendor/navigationd/backup.sqlite",
                            "/data/vendor/navigationd/backup.sqlite",
                            "/data/vendor/navigationd/backup.sqlite",
                            "/data/vendor/navigationd/settings.sqlite",
                            "/data/vendor/navigationd/keystore.sqlite"
                        ).random(),
                        bytesWritten = listOf(10uL, 12uL, 8uL, 8uL).random(),
                        beginTimeStamp = Clock.System.now(),
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }

    private fun sendMsgMockEvents(emissionDelayBoundMillis: Int) = flow {
        var ctr = 0
        var multiplier = 1.3

        while (true) {

            val next = (Random.nextLong(40_000_000) * multiplier).toLong()

            configuration.sysSendmsg?.entries?.keys?.forEach {
                emit(
                    Event.SysSendmsg(
                        pid = it,
                        tid = it + 1u,
                        duration = 10_000_000.nanoseconds + next.nanoseconds,
                        beginTimeStamp = Clock.System.now(),
                    )
                )
            }
            ctr++
            if (ctr % 15 == 0) {
                multiplier *= 1.3
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
                        beginTimeStamp = Clock.System.now(),
                        jniMethodName = jniMethodName,
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
                        timeStamp = Clock.System.now(),
                        targetPid = it.toULong(),
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }

    private fun sysFdTrackingMockEvents(emissionDelayBoundMillis: Int) = flow {
        while (true) {

            val rnd1 = Random.nextFloat()

            if (rnd1 >= 0.3f) {
                configuration.sysFdTracking?.pids?.forEach {
                    val rnd = Random.nextFloat()
                    val syFdMethod =
                        if (rnd > 0.20f) Event.SysFdTracking.SysFdAction.Created
                        else Event.SysFdTracking.SysFdAction.Destroyed
                    emit(
                        Event.SysFdTracking(
                            pid = it,
                            tid = it + 1u,
                            timeStamp = Clock.System.now(),
                            fdAction = syFdMethod,
                        )
                    )
                }
                delay((Random.nextFloat() * (emissionDelayBoundMillis / 5)).toLong())
            } else {
                configuration.sysFdTracking?.pids?.forEach {
                    val rnd = Random.nextFloat()
                    val syFdMethod =
                        if (rnd > 0.40f) Event.SysFdTracking.SysFdAction.Created
                        else Event.SysFdTracking.SysFdAction.Destroyed
                    emit(
                        Event.SysFdTracking(
                            pid = it,
                            tid = it + 1u,
                            timeStamp = Clock.System.now(),
                            fdAction = syFdMethod,
                        )
                    )
                }
                delay((Random.nextFloat() * (emissionDelayBoundMillis)).toLong())
            }
        }
    }

    private fun gcMockEvents(emissionDelayBoundMillis: Int) = flow {
        val prev = 5_054_000UL
        while (true) {
            configuration.gc?.pids?.forEach {
                val diff = -1_000_000L + Random.nextLong(2_000_000L)
                val newNumBytesAllocated = (prev.toLong() + diff).coerceAtLeast(0).toULong()
                val diff2 = -500_000L + Random.nextLong(300_000)
                val newTargetFootprint =
                    (newNumBytesAllocated.toLong() * 2 + diff2).coerceAtLeast(0).toULong()
                val bytesFreed = Random.nextLong(2_000_000L)
                emit(
                    Event.Gc(
                        pid = it,
                        tid = it + 1u,
                        numBytesAllocated = newNumBytesAllocated,
                        targetFootprint = newTargetFootprint,
                        gcsCompleted = 0u,
                        gcCause = 0u,
                        durationNs = 0u,
                        freedObjects = 0u,
                        freedBytes = bytesFreed,
                        freedLosObjects = 0u,
                        freedLosBytes = 0,
                        pauseTimes = listOf(),
                    )
                )
            }
            delay((Random.nextFloat() * emissionDelayBoundMillis).toLong())
        }
    }
}

class RustClientFactory(val url: String) : ClientFactory {
    override suspend fun connect(): Client {
        return RustClient
    }
}
