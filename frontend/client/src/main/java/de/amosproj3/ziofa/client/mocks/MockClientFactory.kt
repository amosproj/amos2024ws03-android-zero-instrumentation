package de.amosproj3.ziofa.client.mocks

import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory

class MockClientFactory : ClientFactory {
    override suspend fun connect(): Client {
        return MockClient
    }
}