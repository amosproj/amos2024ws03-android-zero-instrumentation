// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.events

import kotlinx.coroutines.flow.Flow

interface DataStreamProvider {
    suspend fun counter(ebpfProgramName: String): Flow<UInt>

    suspend fun vfsWriteEvents(pids: List<UInt>?): Flow<BackendEvent.VfsWriteEvent>

    suspend fun sendMessageEvents(pids: List<UInt>?): Flow<BackendEvent.SendMessageEvent>
}
