// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.datetime.Instant
import uniffi.client.jniMethodNameFromI32
import uniffi.client.fileDescriptorChangeOpFromI32

import uniffi.shared.Cmd
import uniffi.shared.EventData
import uniffi.shared.FileDescriptorOp
import uniffi.shared.Filter
import uniffi.shared.JniMethodName
import uniffi.shared.LogEventData
import uniffi.shared.MissingBehavior
import uniffi.shared.UInt32Filter
import kotlin.time.Duration
import kotlin.time.Duration.Companion.nanoseconds

private fun uniffi.shared.Process.into() =
    Process(
        pid,
        ppid,
        state,
        cmd =
        when (val c = cmd) {
            is Cmd.Comm -> Command.Comm(c.v1)
            is Cmd.Cmdline -> Command.Cmdline(c.v1.args)
            null -> null
        },
    )

private fun uniffi.shared.Event.into() =
    when (val d = eventData) {
        is EventData.TimeSeries -> null
        is EventData.Log -> run {
            val context = d.v1.context ?: return null
            val data = d.v1.logEventData ?: return null
            when (data) {
                is LogEventData.Blocking -> Event.SysSendmsg(
                    pid = context.pid,
                    tid = context.tid,
                    beginTimeStamp = context.timestamp.into(),
                    duration = data.v1.duration.into(),
                )

                is LogEventData.FileDescriptorChange -> Event.SysFdTracking(
                    pid = context.pid,
                    tid = context.tid,
                    timeStamp = context.timestamp.into(),
                    fdAction = when (fileDescriptorChangeOpFromI32(data.v1.operation)) {
                        FileDescriptorOp.OPEN -> Event.SysFdTracking.SysFdAction.Created
                        FileDescriptorOp.CLOSE -> Event.SysFdTracking.SysFdAction.Destroyed
                        FileDescriptorOp.UNDEFINED -> Event.SysFdTracking.SysFdAction.Undefined
                    }
                )

                is LogEventData.GarbageCollect -> Event.Gc(
                    pid = context.pid,
                    tid = context.tid,
                    numBytesAllocated = data.v1.numBytesAllocated,
                    targetFootprint = data.v1.targetFootprint,
                    gcsCompleted = data.v1.gcsCompleted,
                    gcCause = data.v1.gcCause,
                    durationNs = data.v1.durationNs,
                    freedObjects = data.v1.freedObjects,
                    freedBytes = data.v1.freedBytes,
                    freedLosObjects = data.v1.freedLosObjects,
                    freedLosBytes = data.v1.freedLosBytes,
                    pauseTimes = data.v1.pauseTimes
                )
                is LogEventData.JniReferences -> Event.JniReferences(
                    pid = context.pid,
                    tid = context.tid,
                    beginTimeStamp = context.timestamp.into(),
                    jniMethodName = when (jniMethodNameFromI32(data.v1.methodName)) {
                        JniMethodName.ADD_LOCAL_REF -> Event.JniReferences.JniMethodName.AddLocalRef
                        JniMethodName.DELETE_LOCAL_REF -> Event.JniReferences.JniMethodName.DeleteLocalRef
                        JniMethodName.ADD_GLOBAL_REF -> Event.JniReferences.JniMethodName.AddGlobalRef
                        JniMethodName.DELETE_GLOBAL_REF -> Event.JniReferences.JniMethodName.DeleteGlobalRef
                        JniMethodName.UNDEFINED -> Event.JniReferences.JniMethodName.Undefined
                    }
                )
                is LogEventData.Signal -> Event.SysSigquit(
                    pid = context.pid,
                    tid = context.tid,
                    timeStamp = context.timestamp.into(),
                    targetPid = data.v1.targetPid.toULong()
                )
                is LogEventData.Write -> Event.VfsWrite(
                    pid = context.pid,
                    tid = context.tid,
                    beginTimeStamp = context.timestamp.into(),
                    fp = data.v1.filePath,
                    bytesWritten = data.v1.bytesWritten
                )
            }
        }

        null -> null
    }

private inline fun <reified T> uniffi.shared.Timestamp?.into(): T =
    when (T::class.java) {
        Instant::class.java -> this
            ?.let { Instant.fromEpochMilliseconds(it.seconds * 1000 + it.nanos / 1_000_000) as T }
            ?: Instant.DISTANT_PAST as T

        else -> {
            throw Exception()
        }
    }

private fun uniffi.shared.Duration?.into() =
    this?.let { it.nanos.nanoseconds } ?: Duration.ZERO

private fun uniffi.shared.Configuration.into() =
    Configuration(
        vfsWrite = writeConfig?.let { VfsWriteConfig(pids = it.filter.toPidList()) },
        sysSendmsg =
        blockingConfig?.let { config ->
            config.filter.toPidList().let {
                SysSendmsgConfig(
                    entries = it.associateWith { config.threshold ?: 32_000_000UL }
                )
            }
        },
        uprobes =
        uprobeConfigs.map {
            UprobeConfig(
                fnName = it.fnName,
                offset = it.offset,
                target = it.target,
                pid = it.pid,
            )
        },
        jniReferences = jniReferencesConfig?.let { JniReferencesConfig(it.filter.toPidList()) },
        sysSigquit = signalConfig?.let { SysSigquitConfig(pids = it.filter.toPidList()) },
        gc = garbageCollectConfig?.let { GcConfig(it.filter.toPidList().toSet()) },
        sysFdTracking =
        fileDescriptorChangeConfig?.let { SysFdTrackingConfig(it.filter.toPidList()) },
    )

private fun Configuration.into() =
    uniffi.shared.Configuration(
        writeConfig = vfsWrite?.let { uniffi.shared.WriteConfig(it.pids.toPidFilter()) },
        blockingConfig =
        sysSendmsg?.let {
            uniffi.shared.BlockingConfig(
                it.entries.keys.toPidFilter(),
                it.entries.values.firstOrNull() ?: 32_000_000U,
            )
        },
        uprobeConfigs =
        uprobes.map {
            uniffi.shared.UprobeConfig(
                fnName = it.fnName,
                offset = it.offset,
                target = it.target,
                pid = it.pid,
            )
        },
        jniReferencesConfig =
        jniReferences?.let { uniffi.shared.JniReferencesConfig(it.pids.toPidFilter()) },
        signalConfig = sysSigquit?.let { uniffi.shared.SignalConfig(it.pids.toPidFilter()) },
        garbageCollectConfig =
        gc?.let { uniffi.shared.GarbageCollectConfig(it.pids.toPidFilter()) },
        fileDescriptorChangeConfig =
        sysFdTracking?.let { uniffi.shared.FileDescriptorChangeConfig(it.pids.toPidFilter()) },
    )

private fun Filter?.toPidList() = this?.pidFilter?.match ?: listOf()

private fun Collection<UInt>.toPidFilter() =
    Filter(
        pidFilter =
        UInt32Filter(
            missingBehavior = MissingBehavior.NOT_MATCH.value,
            match = this.toList(),
            notMatch = listOf(),
        ),
        commFilter = null,
        exePathFilter = null,
        cmdlineFilter = null,
    )

private fun uniffi.shared.Symbol.into() = Symbol(method, offset)

class RustClient(private val inner: uniffi.client.Client) : Client {
    override suspend fun listProcesses(): List<Process> = inner.listProcesses().map { it.into() }

    override suspend fun getConfiguration(): Configuration = inner.getConfiguration().into()

    // TODO remove the workarounds
    override suspend fun setConfiguration(configuration: Configuration) =
        inner.setConfiguration(configuration.into())

    override suspend fun indexSymbols() {
        inner.indexSymbols()
    }

    override suspend fun initStream(): Flow<Event> = inner.initStreamFlow().mapNotNull { it.into() }
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

fun uniffi.client.Client.initStreamFlow() = flow {
    initStream().use { stream ->
        while (true) {
            stream.next()?.also { event -> emit(event) } ?: break
        }
    }
}
