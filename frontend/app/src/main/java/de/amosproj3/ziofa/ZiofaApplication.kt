// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import android.app.Application
import de.amosproj3.ziofa.api.ConfigurationAccess
import de.amosproj3.ziofa.api.ProcessListAccess
import de.amosproj3.ziofa.bl.ConfigurationManager
import de.amosproj3.ziofa.client.mocks.MockClientFactory
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.RustClientFactory
import de.amosproj3.ziofa.ui.configuration.ConfigurationViewModel
import de.amosproj3.ziofa.ui.processes.ProcessesViewModel
import de.amosproj3.ziofa.ui.visualization.VisualizationViewModel
import org.koin.android.ext.koin.androidContext
import org.koin.android.ext.koin.androidLogger
import org.koin.core.context.startKoin
import org.koin.core.module.dsl.viewModel
import org.koin.dsl.binds
import org.koin.dsl.module
import timber.log.Timber

class ZiofaApplication : Application() {

    val appModule = module {
        single<ClientFactory> {
            if (BuildConfig.FLAVOR == "mockedBackend") {
                MockClientFactory()
            } else {
                RustClientFactory("http://[::1]:50051")
            }
        }
        single { ConfigurationManager(clientFactory = get()) } binds arrayOf(
            ConfigurationAccess::class,
            ProcessListAccess::class
        )
        viewModel { ConfigurationViewModel(configurationAccess = get()) }
        viewModel { ProcessesViewModel(processListAccess = get()) }
        viewModel { VisualizationViewModel(clientFactory = get()) }
    }

    override fun onCreate() {
        super.onCreate()
        Timber.plant(Timber.DebugTree()) // start Timber logging

        startKoin {
            androidLogger()
            androidContext(this@ZiofaApplication)
            modules(appModule)
        }
    }
}
