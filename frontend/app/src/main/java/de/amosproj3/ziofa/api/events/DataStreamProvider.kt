// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.events

import kotlinx.coroutines.flow.Flow

interface DataStreamProvider {

    fun vfsWriteEvents(pids: List<UInt>?): Flow<BackendEvent.VfsWriteEvent>

    fun sendMessageEvents(pids: List<UInt>?): Flow<BackendEvent.SendMessageEvent>
}
