/**
 * MCP Settings Component
 *
 * Settings UI for MCP integration configuration.
 */

import { Setting, Notice } from 'obsidian';
import type LabeledAnnotations from '../../main';
import { MCPClient } from '../../mcp';

interface Props {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
}

// Store MCP client instance for testing
let testClient: MCPClient | null = null;

export const MCPSettings = ({ plugin, containerEl }: Props) => {
    const settings = plugin.settings.getValue();
    const mcpConfig = settings.mcp;

    // Header
    containerEl.createEl('h2', { text: 'MCP Integration' });
    containerEl.createEl('p', {
        text: 'Connect to the doc-doctor-mcp server for document operations. Install via: cargo install doc-doctor',
        cls: 'setting-item-description',
    });

    // Enable MCP
    new Setting(containerEl)
        .setName('Enable MCP')
        .setDesc('Use MCP server for document operations (recommended for full functionality)')
        .addToggle((toggle) => {
            toggle.setValue(mcpConfig.enabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'MCP_SET_ENABLED',
                    payload: { enabled: value },
                });
                // Refresh settings display
                containerEl.empty();
                MCPSettings({ plugin, containerEl });
            });
        });

    // Only show remaining settings if enabled
    if (!mcpConfig.enabled) {
        return;
    }

    // Binary path
    const binaryPathSetting = new Setting(containerEl)
        .setName('Binary Path')
        .setDesc('Path to dd-mcp binary (leave empty for auto-detect)');

    binaryPathSetting.addText((text) => {
        text.inputEl.style.width = '300px';
        text.setPlaceholder('Auto-detect (~/.cargo/bin/dd-mcp)')
            .setValue(mcpConfig.binaryPath)
            .onChange((value) => {
                plugin.settings.dispatch({
                    type: 'MCP_SET_BINARY_PATH',
                    payload: { path: value.trim() },
                });
            });
    });

    // Test connection button
    binaryPathSetting.addButton((button) => {
        button
            .setButtonText('Test')
            .setCta()
            .onClick(async () => {
                button.setButtonText('Testing...');
                button.setDisabled(true);

                try {
                    // Clean up any existing test client
                    if (testClient) {
                        await testClient.disconnect();
                        testClient = null;
                    }

                    // Create new test client
                    const currentSettings = plugin.settings.getValue();
                    testClient = new MCPClient({
                        binaryPath: currentSettings.mcp.binaryPath,
                        timeout: currentSettings.mcp.connectionTimeout,
                    });

                    await testClient.connect();
                    const tools = await testClient.listTools();

                    button.setButtonText('Success!');
                    new Notice(`MCP connected! ${tools.length} tools available.`);

                    // Disconnect after test
                    await testClient.disconnect();
                    testClient = null;

                    setTimeout(() => button.setButtonText('Test'), 2000);
                } catch (error) {
                    button.setButtonText('Failed');
                    const message = error instanceof Error ? error.message : 'Unknown error';
                    new Notice(`MCP connection failed: ${message}`);
                    console.error('[Doc Doctor] MCP test error:', error);
                    setTimeout(() => button.setButtonText('Test'), 2000);
                } finally {
                    button.setDisabled(false);
                }
            });
    });

    // Auto-connect
    new Setting(containerEl)
        .setName('Auto-connect on Load')
        .setDesc('Automatically connect to MCP server when plugin loads')
        .addToggle((toggle) => {
            toggle.setValue(mcpConfig.autoConnect).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'MCP_SET_AUTO_CONNECT',
                    payload: { enabled: value },
                });
            });
        });

    // Connection timeout
    new Setting(containerEl)
        .setName('Connection Timeout')
        .setDesc('Maximum time to wait for MCP responses (seconds)')
        .addSlider((slider) => {
            slider
                .setLimits(5, 120, 5)
                .setValue(mcpConfig.connectionTimeout / 1000)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'MCP_SET_CONNECTION_TIMEOUT',
                        payload: { timeout: value * 1000 },
                    });
                });
        });

    // Show status bar
    new Setting(containerEl)
        .setName('Show Status in Status Bar')
        .setDesc('Display MCP connection status in the status bar')
        .addToggle((toggle) => {
            toggle.setValue(mcpConfig.showStatusBar).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'MCP_SET_SHOW_STATUS_BAR',
                    payload: { enabled: value },
                });
            });
        });

    // Installation help
    containerEl.createEl('h4', { text: 'Installation' });
    containerEl.createEl('p', {
        text: 'To install the MCP server, run:',
        cls: 'setting-item-description',
    });

    const codeEl = containerEl.createEl('pre', {
        cls: 'doc-doctor-code-block',
    });
    codeEl.createEl('code', {
        text: 'cargo install doc-doctor',
    });

    containerEl.createEl('p', {
        text: 'The binary will be installed to ~/.cargo/bin/dd-mcp which is auto-detected.',
        cls: 'setting-item-description',
    });

    // Available tools info
    containerEl.createEl('h4', { text: 'Available Operations' });

    const toolsList = containerEl.createEl('ul', {
        cls: 'setting-item-description',
    });

    const tools = [
        'parse_document - Parse frontmatter and content',
        'analyze_document - Full document analysis with health metrics',
        'add_stub / resolve_stub - Manage document stubs',
        'link_stub_anchor - Connect stubs to inline anchors',
        'calculate_health - Get document health score',
        'scan_vault - Analyze multiple documents',
    ];

    tools.forEach((tool) => {
        toolsList.createEl('li', { text: tool });
    });

    // ==========================================================================
    // INTEGRATIONS DIAGNOSTICS
    // ==========================================================================

    containerEl.createEl('h2', { text: 'Plugin Integrations' });
    containerEl.createEl('p', {
        text: 'Check status of optional plugin integrations that enhance Doc Doctor functionality.',
        cls: 'setting-item-description',
    });

    // Obsidian Git diagnostic
    new Setting(containerEl)
        .setName('Obsidian Git')
        .setDesc('Used for version control and sync status')
        .addButton((button) => {
            button
                .setButtonText('Check')
                .onClick(async () => {
                    button.setButtonText('Checking...');
                    button.setDisabled(true);

                    try {
                        // Check if Obsidian Git plugin is installed and enabled
                        const obsidianGit = (plugin.app as any).plugins?.plugins?.['obsidian-git'];

                        if (!obsidianGit) {
                            button.setButtonText('Not Found');
                            new Notice('Obsidian Git plugin is not installed or enabled.');
                        } else if (!obsidianGit._loaded) {
                            button.setButtonText('Disabled');
                            new Notice('Obsidian Git plugin is installed but not enabled.');
                        } else {
                            button.setButtonText('Available');
                            new Notice('Obsidian Git is available and running.');
                        }

                        setTimeout(() => button.setButtonText('Check'), 2000);
                    } catch (error) {
                        button.setButtonText('Error');
                        console.error('[Doc Doctor] Obsidian Git check error:', error);
                        setTimeout(() => button.setButtonText('Check'), 2000);
                    } finally {
                        button.setDisabled(false);
                    }
                });
        })
        .addExtraButton((button) => {
            button
                .setIcon('external-link')
                .setTooltip('Install Obsidian Git')
                .onClick(() => {
                    window.open('obsidian://show-plugin?id=obsidian-git', '_blank');
                });
        });

    // Smart Connections diagnostic
    new Setting(containerEl)
        .setName('Smart Connections')
        .setDesc('Used for finding semantically related notes')
        .addButton((button) => {
            button
                .setButtonText('Check')
                .onClick(async () => {
                    button.setButtonText('Checking...');
                    button.setDisabled(true);

                    try {
                        // Use the SmartConnectionsService for accurate detection
                        if (plugin.smartConnectionsService) {
                            // Force refresh the API reference
                            plugin.smartConnectionsService.refreshApiReference();
                            const status = plugin.smartConnectionsService.getStatus();

                            if (status.smartConnections) {
                                button.setButtonText('Available');
                                const embeddingsInfo = status.embeddingsCount > 0
                                    ? ` (${status.embeddingsCount} embeddings)`
                                    : '';
                                new Notice(`Smart Connections is available${embeddingsInfo}`);
                            } else if (status.error) {
                                button.setButtonText('Limited');
                                new Notice(status.error);
                            } else {
                                button.setButtonText('Fallback');
                                new Notice('Using keyword fallback (Smart Connections unavailable)');
                            }
                        } else {
                            // Fallback to direct check if service not initialized
                            const smartConnections = (plugin.app as any).plugins?.plugins?.['smart-connections'];

                            if (!smartConnections) {
                                button.setButtonText('Not Found');
                                new Notice('Smart Connections plugin is not installed.');
                            } else if (!smartConnections._loaded) {
                                button.setButtonText('Disabled');
                                new Notice('Smart Connections plugin is installed but not enabled.');
                            } else if (!smartConnections.env?.smart_sources) {
                                button.setButtonText('Limited');
                                new Notice('Smart Connections is loading - try again in a moment.');
                            } else {
                                button.setButtonText('Available');
                                new Notice('Smart Connections is available.');
                            }
                        }

                        setTimeout(() => button.setButtonText('Check'), 2000);
                    } catch (error) {
                        button.setButtonText('Error');
                        console.error('[Doc Doctor] Smart Connections check error:', error);
                        setTimeout(() => button.setButtonText('Check'), 2000);
                    } finally {
                        button.setDisabled(false);
                    }
                });
        })
        .addExtraButton((button) => {
            button
                .setIcon('external-link')
                .setTooltip('Install Smart Connections')
                .onClick(() => {
                    window.open('obsidian://show-plugin?id=smart-connections', '_blank');
                });
        });

    // Dataview diagnostic (commonly used)
    new Setting(containerEl)
        .setName('Dataview')
        .setDesc('Used for querying document metadata')
        .addButton((button) => {
            button
                .setButtonText('Check')
                .onClick(async () => {
                    button.setButtonText('Checking...');
                    button.setDisabled(true);

                    try {
                        const dataview = (plugin.app as any).plugins?.plugins?.['dataview'];

                        if (!dataview) {
                            button.setButtonText('Not Found');
                            new Notice('Dataview plugin is not installed or enabled.');
                        } else if (!dataview._loaded) {
                            button.setButtonText('Disabled');
                            new Notice('Dataview plugin is installed but not enabled.');
                        } else {
                            const api = (plugin.app as any).plugins?.plugins?.dataview?.api;
                            if (api) {
                                button.setButtonText('Available');
                                new Notice('Dataview is available with API access.');
                            } else {
                                button.setButtonText('Limited');
                                new Notice('Dataview is running but API may not be fully available.');
                            }
                        }

                        setTimeout(() => button.setButtonText('Check'), 2000);
                    } catch (error) {
                        button.setButtonText('Error');
                        console.error('[Doc Doctor] Dataview check error:', error);
                        setTimeout(() => button.setButtonText('Check'), 2000);
                    } finally {
                        button.setDisabled(false);
                    }
                });
        })
        .addExtraButton((button) => {
            button
                .setIcon('external-link')
                .setTooltip('Install Dataview')
                .onClick(() => {
                    window.open('obsidian://show-plugin?id=dataview', '_blank');
                });
        });

    // Run all diagnostics button
    new Setting(containerEl)
        .setName('Run All Diagnostics')
        .setDesc('Check all integrations at once')
        .addButton((button) => {
            button
                .setButtonText('Run All')
                .setCta()
                .onClick(async () => {
                    button.setButtonText('Running...');
                    button.setDisabled(true);

                    const results: string[] = [];
                    const plugins = (plugin.app as any).plugins?.plugins || {};

                    // Check MCP
                    if (mcpConfig.enabled) {
                        try {
                            const client = new MCPClient({
                                binaryPath: mcpConfig.binaryPath,
                                timeout: 5000,
                            });
                            await client.connect();
                            await client.disconnect();
                            results.push('MCP: OK');
                        } catch {
                            results.push('MCP: Failed');
                        }
                    } else {
                        results.push('MCP: Disabled');
                    }

                    // Check plugins
                    const checkPlugin = (id: string, name: string) => {
                        const p = plugins[id];
                        if (!p) {
                            results.push(`${name}: Not installed`);
                        } else if (!p._loaded) {
                            results.push(`${name}: Disabled`);
                        } else {
                            results.push(`${name}: OK`);
                        }
                    };

                    checkPlugin('obsidian-git', 'Obsidian Git');
                    checkPlugin('smart-connections', 'Smart Connections');
                    checkPlugin('dataview', 'Dataview');

                    // Show results
                    new Notice(results.join('\n'), 5000);
                    console.log('[Doc Doctor] Diagnostics:', results);

                    button.setButtonText('Done!');
                    setTimeout(() => button.setButtonText('Run All'), 2000);
                    button.setDisabled(false);
                });
        });
};
