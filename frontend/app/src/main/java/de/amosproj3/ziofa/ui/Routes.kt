// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui

/** Routes for the main navigation */
enum class Routes {
    /** Visualization screen */
    Visualize,

    /**
     * Configuration screen with options for the process, requires parameters for display name and
     * pids
     */
    IndividualConfiguration,

    /** Home screen */
    Home,

    /** Processes screen for selecting processes to configure */
    Processes,

    /**
     * Configuration menu that allows to navigate to [IndividualConfiguration] for a process or a
     * global configuration
     */
    Configuration,

    /** Symbols search screen for setting arbitrary uprobes */
    Symbols,

    /** Screen to reset the configuration */
    Reset,

    /** Init loading screen for bootstrapping or error display */
    Init,
}
