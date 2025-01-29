// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa

import android.app.Application
import android.content.Context
import android.content.pm.PackageManager
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import de.amosproj3.ziofa.api.events.DataStreamProvider
import de.amosproj3.ziofa.api.overlay.OverlayController
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.RustClientFactory
import de.amosproj3.ziofa.platform.configuration.ConfigurationManager
import de.amosproj3.ziofa.platform.configuration.UProbeManager
import de.amosproj3.ziofa.platform.events.DataStreamManager
import de.amosproj3.ziofa.platform.overlay.OverlayManager
import de.amosproj3.ziofa.platform.processes.PackageInformationProvider
import de.amosproj3.ziofa.platform.processes.RunningComponentsProvider
import de.amosproj3.ziofa.ui.configuration.ConfigurationViewModel
import de.amosproj3.ziofa.ui.init.InitViewModel
import de.amosproj3.ziofa.ui.overlay.OverlayViewModel
import de.amosproj3.ziofa.ui.processes.ProcessesViewModel
import de.amosproj3.ziofa.ui.reset.ResetViewModel
import de.amosproj3.ziofa.ui.symbols.SymbolsViewModel
import de.amosproj3.ziofa.ui.visualization.VisualizationViewModel
import kotlinx.coroutines.CoroutineScope
import org.koin.android.ext.koin.androidContext
import org.koin.android.ext.koin.androidLogger
import org.koin.core.context.startKoin
import org.koin.core.logger.Level
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.core.parameter.parametersOf
import org.koin.dsl.module
import timber.log.Timber

class ZiofaApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Timber.plant(Timber.DebugTree()) // start Timber logging

        startKoin {
            androidLogger(level = Level.DEBUG)
            androidContext(this@ZiofaApplication)
            modules(
                module {
                    createExternalDependencies()
                    createBLModules()
                    createViewModelFactories()
                }
            )
        }
    }

    private fun Module.createExternalDependencies() {
        single<ClientFactory> { RustClientFactory("http://[::1]:50051") }
        single<PackageManager> { get<Context>().packageManager }
    }

    private fun Module.createBLModules() {
        single { PackageInformationProvider(get()) }
        single<RunningComponentsAccess> {
            RunningComponentsProvider(clientFactory = get(), packageInformationProvider = get())
        }
        single<ConfigurationAccess> { ConfigurationManager(clientFactory = get()) }
        factory<DataStreamProvider> { (scope: CoroutineScope) -> DataStreamManager(get(), scope) }
        single<SymbolsAccess> { UProbeManager(get()) }
        single<OverlayController> { OverlayManager(get()) }
    }

    private fun Module.createViewModelFactories() {
        viewModel { InitViewModel(get()) }
        viewModel { (pids: List<UInt>) ->
            ConfigurationViewModel(configurationAccess = get(), pids = pids)
        }
        viewModel { ResetViewModel(get()) }
        viewModel { ProcessesViewModel(runningComponentsProvider = get()) }
        viewModel {
            VisualizationViewModel(
                configurationAccess = get(),
                runningComponentsAccess = get(),
                dataStreamProviderFactory = { get { parametersOf(it) } },
                overlayController = get(),
            )
        }

        viewModel { (pids: List<UInt>) -> SymbolsViewModel(get(), get(), pids) }

        viewModel {
            OverlayViewModel(
                overlayManager = get(),
                dataStreamProviderFactory = { get { parametersOf(it) } },
            )
        }
    }
}
