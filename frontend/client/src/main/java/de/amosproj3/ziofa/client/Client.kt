// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow

data class Configuration(
    val vfsWrite: VfsWriteConfig?,
    val sysSendmsg: SysSendmsgConfig?,
    val uprobes: List<UprobeConfig>,
)

data class VfsWriteConfig(val pids: List<UInt>)

data class SysSendmsgConfig(val pids: List<UInt>)

data class UprobeConfig(val fnName: String, val offset: ULong, var target: String, val pid: Int?)

sealed class Event {
    data class VfsWrite(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: ULong,
        val fp: ULong,
        val bytesWritten: ULong,
    ) : Event()

    data class SysSendmsg(
        val pid: UInt,
        val tid: UInt,
        val beginTimeStamp: ULong,
        val fd: ULong,
        val durationNanoSecs: ULong,
    ) : Event()
}

data class Process(val pid: Int, val ppid: Int, val state: String, val cmd: Command?)

sealed class Command {
    data class Cmdline(val components: List<String>) : Command()

    data class Comm(val name: String) : Command()
}

interface Client {
    suspend fun serverCount(): Flow<UInt>

    suspend fun load()

    suspend fun attach(iface: String)

    suspend fun unload()

    suspend fun detach(iface: String)

    suspend fun startCollecting()

    suspend fun stopCollecting()

    suspend fun checkServer()

    suspend fun listProcesses(): List<Process>

    suspend fun getConfiguration(): Configuration

    suspend fun setConfiguration(configuration: Configuration)

    suspend fun initStream(): Flow<Event>
}

interface ClientFactory {
    suspend fun connect(): Client
}
