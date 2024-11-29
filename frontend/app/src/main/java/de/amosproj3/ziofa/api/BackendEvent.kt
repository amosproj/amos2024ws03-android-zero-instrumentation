// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api

sealed class BackendEvent(
    val fileDescriptor: ULong,
    val processId: UInt,
    val startTimestamp: ULong,
    val durationOrSize: ULong,
) {

    data class VfsWriteEvent(
        val fd: ULong,
        val pid: UInt,
        val size: ULong,
        val timestampMillis: ULong, // unix time
    ) : BackendEvent(fd, pid, timestampMillis, size)

    data class SendMessageEvent(
        val fd: ULong,
        val pid: UInt,
        val tid: UInt,
        val beginTimestamp: ULong,
        val durationNanos: ULong,
    ) : BackendEvent(fd, pid, beginTimestamp, durationNanos)
}
