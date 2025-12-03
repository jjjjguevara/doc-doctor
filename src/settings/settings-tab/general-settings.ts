/**
 * General Settings Component
 *
 * Central settings tab with feature toggles, API configuration, and unified diagnostics.
 */

import { Notice, Setting } from 'obsidian';
import type LabeledAnnotations from '../../main';
import { AVAILABLE_MODELS, LLMProvider } from '../../llm/llm-types';

interface Props {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
}

export const GeneralSettings = ({ plugin, containerEl }: Props) => {
    const settings = plugin.settings.getValue();

    // ==========================================================================
    // FEATURE TOGGLES
    // ==========================================================================

    containerEl.createEl('h2', { text: 'Features' });
    containerEl.createEl('p', {
        text: 'Enable or disable major plugin features. Disabled features will not load on startup.',
        cls: 'setting-item-description',
    });

    // Annotations toggle
    new Setting(containerEl)
        .setName('Annotations')
        .setDesc('Highlight and annotate text with labels and comments')
        .addToggle((toggle) => {
            toggle.setValue(settings.features.annotations).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_FEATURE_ANNOTATIONS',
                    payload: { enabled: value },
                });
                new Notice(`Annotations ${value ? 'enabled' : 'disabled'}. Restart Obsidian to apply.`);
            });
        });

    // Stubs toggle
    new Setting(containerEl)
        .setName('Stubs')
        .setDesc('Track document gaps, TODOs, and incomplete sections')
        .addToggle((toggle) => {
            toggle.setValue(settings.features.stubs).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_FEATURE_STUBS',
                    payload: { enabled: value },
                });
                new Notice(`Stubs ${value ? 'enabled' : 'disabled'}. Restart Obsidian to apply.`);
            });
        });

    // AI toggle
    new Setting(containerEl)
        .setName('AI')
        .setDesc('LLM-powered document analysis and stub suggestions')
        .addToggle((toggle) => {
            toggle.setValue(settings.features.ai).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_FEATURE_AI',
                    payload: { enabled: value },
                });
                new Notice(`AI ${value ? 'enabled' : 'disabled'}. Restart Obsidian to apply.`);
            });
        });

    // Explore toggle
    new Setting(containerEl)
        .setName('Explore')
        .setDesc('Semantic search and related notes via Smart Connections')
        .addToggle((toggle) => {
            toggle.setValue(settings.features.explore).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_FEATURE_EXPLORE',
                    payload: { enabled: value },
                });
                new Notice(`Explore ${value ? 'enabled' : 'disabled'}. Restart Obsidian to apply.`);
            });
        });

    // ==========================================================================
    // API CONFIGURATION
    // ==========================================================================

    containerEl.createEl('h2', { text: 'API Configuration' });

    const llmConfig = settings.llm;

    // Provider selection
    new Setting(containerEl)
        .setName('LLM Provider')
        .setDesc('Select your AI provider for document analysis')
        .addDropdown((dropdown) => {
            dropdown
                .addOption('anthropic', 'Anthropic (Claude)')
                .addOption('openai', 'OpenAI (GPT)')
                .setValue(llmConfig.provider)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_PROVIDER',
                        payload: { provider: value as LLMProvider },
                    });
                    // Refresh settings display to update model options
                    containerEl.empty();
                    GeneralSettings({ plugin, containerEl });
                });
        });

    // API Key
    const apiKeySetting = new Setting(containerEl)
        .setName('API Key')
        .setDesc('Your API key for the selected LLM provider');

    apiKeySetting.addText((text) => {
        text.inputEl.type = 'password';
        text.inputEl.style.width = '250px';
        text.setPlaceholder('Enter API key...')
            .setValue(llmConfig.apiKey)
            .onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_API_KEY',
                    payload: { apiKey: value.trim() },
                });
            });
    });

    apiKeySetting.addButton((button) => {
        let isVisible = false;
        button.setButtonText('Show').onClick(() => {
            const input = apiKeySetting.controlEl.querySelector('input');
            if (input) {
                isVisible = !isVisible;
                input.type = isVisible ? 'text' : 'password';
                button.setButtonText(isVisible ? 'Hide' : 'Show');
            }
        });
    });

    // Model selection
    const models = AVAILABLE_MODELS[llmConfig.provider];
    new Setting(containerEl)
        .setName('Model')
        .setDesc('AI model for analysis')
        .addDropdown((dropdown) => {
            models.forEach((model) => {
                const label = model.recommended ? `${model.name} (recommended)` : model.name;
                dropdown.addOption(model.id, label);
            });
            dropdown.setValue(llmConfig.model).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_MODEL',
                    payload: { model: value },
                });
            });
        });

    // Firecrawl API Key (for web search)
    const firecrawlSetting = new Setting(containerEl)
        .setName('Firecrawl API Key')
        .setDesc('Optional: Enable web search and URL scraping for external context');

    firecrawlSetting.addText((text) => {
        text.inputEl.type = 'password';
        text.inputEl.style.width = '250px';
        text.setPlaceholder('Enter Firecrawl key...')
            .setValue(llmConfig.firecrawl?.apiKey || '')
            .onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_FIRECRAWL_API_KEY',
                    payload: { apiKey: value.trim() },
                });
            });
    });

    firecrawlSetting.addButton((button) => {
        let isVisible = false;
        button.setButtonText('Show').onClick(() => {
            const input = firecrawlSetting.controlEl.querySelector('input');
            if (input) {
                isVisible = !isVisible;
                input.type = isVisible ? 'text' : 'password';
                button.setButtonText(isVisible ? 'Hide' : 'Show');
            }
        });
    });

    // ==========================================================================
    // UNIFIED DIAGNOSTICS
    // ==========================================================================

    containerEl.createEl('h2', { text: 'Diagnostics' });
    containerEl.createEl('p', {
        text: 'Test connections to external services and plugins.',
        cls: 'setting-item-description',
    });

    // LLM Connection Test
    const llmDiagSetting = new Setting(containerEl)
        .setName('LLM Connection');

    const llmStatusEl = llmDiagSetting.descEl.createEl('span', { cls: 'diagnostic-status' });
    llmStatusEl.textContent = llmConfig.apiKey ? '○ Not tested' : '○ No API key';

    llmDiagSetting.addButton((button) => {
        button.setButtonText('Test').onClick(async () => {
            button.setButtonText('Testing...');
            button.setDisabled(true);
            llmStatusEl.textContent = '○ Testing...';

            try {
                const { getLLMService } = await import('../../llm/llm-service');
                const currentSettings = plugin.settings.getValue();
                const service = getLLMService(currentSettings.llm, currentSettings.stubs);
                const result = await service.testConnection();

                if (result.success) {
                    llmStatusEl.textContent = '● Connected';
                    llmStatusEl.style.color = 'var(--color-green)';
                } else {
                    llmStatusEl.textContent = `○ Failed: ${result.message}`;
                    llmStatusEl.style.color = 'var(--text-error)';
                }
            } catch (error) {
                llmStatusEl.textContent = '○ Error';
                llmStatusEl.style.color = 'var(--text-error)';
            } finally {
                button.setButtonText('Test');
                button.setDisabled(false);
            }
        });
    });

    // MCP Server Test
    const mcpDiagSetting = new Setting(containerEl)
        .setName('MCP Server');

    const mcpStatusEl = mcpDiagSetting.descEl.createEl('span', { cls: 'diagnostic-status' });
    const mcpConnected = plugin.mcpClient?.isConnected?.() ?? false;
    mcpStatusEl.textContent = mcpConnected ? '● Connected' : '○ Not connected';
    mcpStatusEl.style.color = mcpConnected ? 'var(--color-green)' : '';

    mcpDiagSetting.addButton((button) => {
        button.setButtonText('Test').onClick(async () => {
            button.setButtonText('Testing...');
            button.setDisabled(true);
            mcpStatusEl.textContent = '○ Testing...';

            try {
                if (plugin.mcpClient) {
                    await plugin.mcpClient.connect();
                    // Check if connected after connect attempt
                    const isConnected = plugin.mcpClient.isConnected?.() ?? false;
                    if (isConnected) {
                        mcpStatusEl.textContent = '● Connected';
                        mcpStatusEl.style.color = 'var(--color-green)';
                    } else {
                        mcpStatusEl.textContent = '○ Failed to connect';
                        mcpStatusEl.style.color = 'var(--text-error)';
                    }
                } else {
                    mcpStatusEl.textContent = '○ MCP client not initialized';
                    mcpStatusEl.style.color = 'var(--text-muted)';
                }
            } catch (error) {
                mcpStatusEl.textContent = '○ Error';
                mcpStatusEl.style.color = 'var(--text-error)';
            } finally {
                button.setButtonText('Test');
                button.setDisabled(false);
            }
        });
    });

    // Smart Connections Test
    const scDiagSetting = new Setting(containerEl)
        .setName('Smart Connections');

    const scStatusEl = scDiagSetting.descEl.createEl('span', { cls: 'diagnostic-status' });

    const updateSCStatus = () => {
        if (plugin.smartConnectionsService) {
            const status = plugin.smartConnectionsService.getStatus();
            if (status.smartConnections) {
                scStatusEl.textContent = status.embeddingsCount > 0
                    ? `● Connected (${status.embeddingsCount} embeddings)`
                    : '● Connected';
                scStatusEl.style.color = 'var(--color-green)';
            } else if (status.fallbackMode) {
                scStatusEl.textContent = '○ Fallback mode';
                scStatusEl.style.color = 'var(--text-warning)';
            } else {
                scStatusEl.textContent = status.error || '○ Not available';
                scStatusEl.style.color = 'var(--text-muted)';
            }
        } else {
            scStatusEl.textContent = '○ Service not initialized';
            scStatusEl.style.color = 'var(--text-muted)';
        }
    };
    updateSCStatus();

    scDiagSetting.addButton((button) => {
        button.setButtonText('Refresh').onClick(() => {
            if (plugin.smartConnectionsService) {
                plugin.smartConnectionsService.refreshApiReference();
            }
            updateSCStatus();
        });
    });

    // Plugin Integrations
    containerEl.createEl('h3', { text: 'Plugin Integrations' });

    const plugins = (plugin.app as any).plugins?.plugins;

    // Obsidian Git
    const gitPlugin = plugins?.['obsidian-git'];
    new Setting(containerEl)
        .setName('Obsidian Git')
        .setDesc(gitPlugin ? '● Available' : '○ Not installed')
        .descEl.style.color = gitPlugin ? 'var(--color-green)' : '';

    // Dataview
    const dataviewPlugin = plugins?.['dataview'];
    new Setting(containerEl)
        .setName('Dataview')
        .setDesc(dataviewPlugin ? '● Available' : '○ Not installed')
        .descEl.style.color = dataviewPlugin ? 'var(--color-green)' : '';

    // Run All Diagnostics button
    new Setting(containerEl)
        .addButton((button) => {
            button
                .setButtonText('Run All Diagnostics')
                .setCta()
                .onClick(async () => {
                    button.setButtonText('Running...');
                    button.setDisabled(true);

                    // Trigger all test buttons
                    const testButtons = containerEl.querySelectorAll('button');
                    const testsRun: string[] = [];

                    testButtons.forEach((btn) => {
                        if (btn.textContent === 'Test' || btn.textContent === 'Refresh') {
                            btn.click();
                            testsRun.push(btn.textContent);
                        }
                    });

                    setTimeout(() => {
                        button.setButtonText('Run All Diagnostics');
                        button.setDisabled(false);
                        new Notice(`Ran ${testsRun.length} diagnostic tests`);
                    }, 2000);
                });
        });

    // ==========================================================================
    // DEBUG MODE
    // ==========================================================================

    containerEl.createEl('h2', { text: 'Debug Mode' });

    // Enable debug
    new Setting(containerEl)
        .setName('Enable Debug Logging')
        .setDesc('Show detailed logs in developer console')
        .addToggle((toggle) => {
            toggle.setValue(llmConfig.debug.enabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_DEBUG_ENABLED',
                    payload: { enabled: value },
                });
            });
        });

    // Dry run mode
    new Setting(containerEl)
        .setName('Dry Run Mode')
        .setDesc('Preview prompts without making API calls')
        .addToggle((toggle) => {
            toggle.setValue(llmConfig.debug.dryRunMode).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_DRY_RUN_MODE',
                    payload: { enabled: value },
                });
            });
        });

    // Log level
    new Setting(containerEl)
        .setName('Log Level')
        .setDesc('Verbosity of debug output')
        .addDropdown((dropdown) => {
            dropdown
                .addOption('error', 'Error')
                .addOption('warn', 'Warning')
                .addOption('info', 'Info')
                .addOption('debug', 'Debug')
                .setValue(llmConfig.debug.logLevel)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_DEBUG_LOG_LEVEL',
                        payload: { level: value as 'error' | 'warn' | 'info' | 'debug' },
                    });
                });
        });

    // Add styles for diagnostic status
    const style = document.createElement('style');
    style.textContent = `
        .diagnostic-status {
            margin-left: 8px;
            font-size: var(--font-ui-small);
        }
    `;
    containerEl.appendChild(style);
};
