// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.gradle.rust

import org.gradle.api.NamedDomainObjectProvider
import org.gradle.api.provider.Property

abstract class UniffiSpec {
  abstract val uniffiCli: Property<NamedDomainObjectProvider<Binary>>
}
