// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.client

import kotlinx.coroutines.flow.Flow
import uniffi.shared.Configuration
import uniffi.shared.Event
import uniffi.shared.Process

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
