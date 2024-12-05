// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import kotlinx.coroutines.flow.Flow

interface LocalConfigurationAccess {

    /**
     * Emits updates both unconfirmed changes and confirmed changes (these override the unconfirmed)
     */
    val localConfiguration: Flow<ConfigurationUpdate>

    /**
     * Change the local configuration of a feature. If the feature is PID dependent, this function
     * will enable or disable it ==> for the respective PIDs <== depending on [enable].
     *
     * @param enable Whether to enable or disable the feature.
     * @param vfsWriteFeature A [VfsWriteConfig] update to apply or null if this should not be
     *   changed.
     * @param sendMessageFeature A [SysSendmsgConfig] update to apply or null if this should not be
     *   changed.
     * @param uprobesFeature TODO
     */
    fun changeFeatureConfiguration(
        enable: Boolean,
        vfsWriteFeature: VfsWriteConfig? = null,
        sendMessageFeature: SysSendmsgConfig? = null,
        uprobesFeature: List<UprobeConfig>? = listOf(),
        jniReferencesFeature: JniReferencesConfig? = null,
    )

    /** Submit the local configuration to the backend. */
    fun submitConfiguration()
}
