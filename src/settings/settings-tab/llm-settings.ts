/**
 * LLM Settings Component
 *
 * Settings UI for LLM-powered stub suggestions configuration.
 */

import { Setting } from 'obsidian';
import type LabeledAnnotations from '../../main';
import { AVAILABLE_MODELS, LLMProvider } from '../../llm/llm-types';

interface Props {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
}

export const LLMSettings = ({ plugin, containerEl }: Props) => {
    const settings = plugin.settings.getValue();
    const llmConfig = settings.llm;

    // Header
    containerEl.createEl('h2', { text: 'LLM Configuration' });
    containerEl.createEl('p', {
        text: 'Configure AI-powered stub suggestions. Requires an API key from your chosen provider.',
        cls: 'setting-item-description',
    });

    // Enable LLM
    new Setting(containerEl)
        .setName('Enable LLM Features')
        .setDesc('Enable AI-powered document analysis and stub suggestions')
        .addToggle((toggle) => {
            toggle.setValue(llmConfig.enabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_ENABLED',
                    payload: { enabled: value },
                });
                // Refresh settings display
                containerEl.empty();
                LLMSettings({ plugin, containerEl });
            });
        });

    // Only show remaining settings if enabled
    if (!llmConfig.enabled) {
        return;
    }

    // Provider selection
    new Setting(containerEl)
        .setName('Provider')
        .setDesc('Select your LLM provider')
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
                    LLMSettings({ plugin, containerEl });
                });
        });

    // API Key
    const apiKeySetting = new Setting(containerEl)
        .setName('API Key')
        .setDesc('Your API key for the selected provider');

    apiKeySetting.addText((text) => {
        text.inputEl.type = 'password';
        text.inputEl.style.width = '300px';
        text.setPlaceholder('Enter your API key...')
            .setValue(llmConfig.apiKey)
            .onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_API_KEY',
                    payload: { apiKey: value.trim() },
                });
            });
    });

    // Add show/hide toggle for API key
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

    // Test connection button
    apiKeySetting.addButton((button) => {
        button
            .setButtonText('Test')
            .setCta()
            .onClick(async () => {
                button.setButtonText('Testing...');
                button.setDisabled(true);

                try {
                    const { getLLMService } = await import('../../llm/llm-service');
                    const currentSettings = plugin.settings.getValue();
                    const service = getLLMService(currentSettings.llm, currentSettings.stubs);
                    const result = await service.testConnection();

                    if (result.success) {
                        button.setButtonText('Success!');
                        setTimeout(() => button.setButtonText('Test'), 2000);
                    } else {
                        button.setButtonText('Failed');
                        console.error('[Doc Doctor] Connection test failed:', result.message);
                        setTimeout(() => button.setButtonText('Test'), 2000);
                    }
                } catch (error) {
                    button.setButtonText('Error');
                    console.error('[Doc Doctor] Connection test error:', error);
                    setTimeout(() => button.setButtonText('Test'), 2000);
                } finally {
                    button.setDisabled(false);
                }
            });
    });

    // Security warning
    containerEl.createEl('p', {
        text: '⚠️ API keys are stored in plugin settings (not encrypted). Consider using environment variables for sensitive deployments.',
        cls: 'setting-item-description mod-warning',
    });

    // Model selection
    const currentSettings = plugin.settings.getValue();
    const models = AVAILABLE_MODELS[currentSettings.llm.provider];

    new Setting(containerEl)
        .setName('Model')
        .setDesc('Select the AI model to use for analysis')
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

    // Advanced settings header
    containerEl.createEl('h3', { text: 'Advanced Settings' });

    // Insertion order
    new Setting(containerEl)
        .setName('Insertion Order')
        .setDesc('Where to insert new stubs and references in the array')
        .addDropdown((dropdown) => {
            dropdown
                .addOption('bottom', 'Bottom (append)')
                .addOption('top', 'Top (prepend)')
                .setValue(llmConfig.insertionOrder || 'bottom')
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_INSERTION_ORDER',
                        payload: { order: value as 'top' | 'bottom' },
                    });
                });
        });

    // Reference settings header
    containerEl.createEl('h4', { text: 'Reference Properties' });

    // Separate reference properties toggle
    new Setting(containerEl)
        .setName('Separate Vault and Web References')
        .setDesc('Use different frontmatter properties for vault links vs web URLs')
        .addToggle((toggle) => {
            toggle
                .setValue(llmConfig.separateReferenceProperties || false)
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_SEPARATE_REFERENCE_PROPERTIES',
                        payload: { enabled: value },
                    });
                    // Refresh settings display
                    containerEl.empty();
                    LLMSettings({ plugin, containerEl });
                });
        });

    // Vault reference property
    new Setting(containerEl)
        .setName(llmConfig.separateReferenceProperties ? 'Vault Reference Property' : 'Reference Property')
        .setDesc(llmConfig.separateReferenceProperties
            ? 'Frontmatter property for internal vault links'
            : 'Frontmatter property for all references')
        .addText((text) => {
            text.setPlaceholder('references')
                .setValue(llmConfig.vaultReferenceProperty || 'references')
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_VAULT_REFERENCE_PROPERTY',
                        payload: { property: value.trim() || 'references' },
                    });
                });
        });

    // Web reference property (only show if separate properties enabled)
    if (llmConfig.separateReferenceProperties) {
        new Setting(containerEl)
            .setName('Web Reference Property')
            .setDesc('Frontmatter property for external web URLs')
            .addText((text) => {
                text.setPlaceholder('sources')
                    .setValue(llmConfig.webReferenceProperty || 'references')
                    .onChange((value) => {
                        plugin.settings.dispatch({
                            type: 'LLM_SET_WEB_REFERENCE_PROPERTY',
                            payload: { property: value.trim() || 'references' },
                        });
                    });
            });
    }

    // Max tokens
    new Setting(containerEl)
        .setName('Max Tokens')
        .setDesc('Maximum tokens for LLM response (256-16384)')
        .addSlider((slider) => {
            slider
                .setLimits(256, 8192, 256)
                .setValue(llmConfig.maxTokens)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_MAX_TOKENS',
                        payload: { maxTokens: value },
                    });
                });
        });

    // Temperature
    new Setting(containerEl)
        .setName('Temperature')
        .setDesc('Lower = more consistent, higher = more creative (0-1)')
        .addSlider((slider) => {
            slider
                .setLimits(0, 1, 0.1)
                .setValue(llmConfig.temperature)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_TEMPERATURE',
                        payload: { temperature: value },
                    });
                });
        });

    // Timeout
    new Setting(containerEl)
        .setName('Request Timeout')
        .setDesc('Maximum time to wait for response (seconds)')
        .addSlider((slider) => {
            slider
                .setLimits(10, 120, 5)
                .setValue(llmConfig.timeout / 1000)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_TIMEOUT',
                        payload: { timeout: value * 1000 },
                    });
                });
        });

    // Debug settings header
    containerEl.createEl('h3', { text: 'Debugging' });

    // Enable debug
    new Setting(containerEl)
        .setName('Enable Debug Mode')
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
        .setDesc('Preview prompts without making API calls (useful for debugging)')
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

    // Request history size
    new Setting(containerEl)
        .setName('Request History Size')
        .setDesc('Number of recent requests to store for debugging')
        .addSlider((slider) => {
            slider
                .setLimits(1, 20, 1)
                .setValue(llmConfig.debug.historySize)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_DEBUG_HISTORY_SIZE',
                        payload: { size: value },
                    });
                });
        });

    // View request history button (if debug enabled)
    if (llmConfig.debug.enabled) {
        new Setting(containerEl)
            .setName('Request History')
            .setDesc('View recent LLM requests and responses')
            .addButton((button) => {
                button.setButtonText('View History').onClick(async () => {
                    const { getLLMService } = await import('../../llm/llm-service');
                    const currentSettings = plugin.settings.getValue();
                    const service = getLLMService(currentSettings.llm, currentSettings.stubs);
                    const history = service.getRequestHistory();

                    console.group('[Doc Doctor] Request History');
                    history.forEach((entry, i) => {
                        console.group(`Request ${i + 1}: ${entry.documentPath}`);
                        console.log('Timestamp:', entry.timestamp);
                        console.log('Provider:', entry.provider, entry.model);
                        console.log('Token Estimate:', entry.request.tokenEstimate);
                        if (entry.response) {
                            console.log('Duration:', entry.response.duration, 'ms');
                            console.log('Suggestions:', entry.response.parsed.suggested_stubs.length);
                        }
                        if (entry.error) {
                            console.error('Error:', entry.error);
                        }
                        console.groupEnd();
                    });
                    console.groupEnd();

                    // Also show a notice
                    const { Notice } = await import('obsidian');
                    new Notice(`Request history logged to console (${history.length} entries)`);
                });
            })
            .addButton((button) => {
                button.setButtonText('Clear History').onClick(async () => {
                    const { getLLMService } = await import('../../llm/llm-service');
                    const currentSettings = plugin.settings.getValue();
                    const service = getLLMService(currentSettings.llm, currentSettings.stubs);
                    service.clearHistory();

                    const { Notice } = await import('obsidian');
                    new Notice('Request history cleared');
                });
            });
    }

    // ==========================================================================
    // FIRECRAWL SECTION
    // ==========================================================================

    containerEl.createEl('h2', { text: 'External Context (Firecrawl)' });
    containerEl.createEl('p', {
        text: 'Enable web search and URL scraping to enrich LLM analysis with external context.',
        cls: 'setting-item-description',
    });

    const firecrawlConfig = llmConfig.firecrawl;

    // Enable Firecrawl
    new Setting(containerEl)
        .setName('Enable Firecrawl')
        .setDesc('Enable web search and URL scraping during analysis')
        .addToggle((toggle) => {
            toggle.setValue(firecrawlConfig?.enabled || false).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_FIRECRAWL_ENABLED',
                    payload: { enabled: value },
                });
                containerEl.empty();
                LLMSettings({ plugin, containerEl });
            });
        });

    if (!firecrawlConfig?.enabled) {
        return;
    }

    // Firecrawl API Key
    const fcApiKeySetting = new Setting(containerEl)
        .setName('Firecrawl API Key')
        .setDesc('Get your API key from firecrawl.dev');

    fcApiKeySetting.addText((text) => {
        text.inputEl.type = 'password';
        text.inputEl.style.width = '300px';
        text.setPlaceholder('fc-...')
            .setValue(firecrawlConfig.apiKey || '')
            .onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_FIRECRAWL_API_KEY',
                    payload: { apiKey: value.trim() },
                });
            });
    });

    // Show/hide toggle
    fcApiKeySetting.addButton((button) => {
        let isVisible = false;
        button.setButtonText('Show').onClick(() => {
            const input = fcApiKeySetting.controlEl.querySelector('input');
            if (input) {
                isVisible = !isVisible;
                input.type = isVisible ? 'text' : 'password';
                button.setButtonText(isVisible ? 'Hide' : 'Show');
            }
        });
    });

    // Test connection
    fcApiKeySetting.addButton((button) => {
        button
            .setButtonText('Test')
            .setCta()
            .onClick(async () => {
                button.setButtonText('Testing...');
                button.setDisabled(true);

                try {
                    const { FirecrawlService } = await import('../../llm/firecrawl-service');
                    const currentSettings = plugin.settings.getValue();
                    const service = new FirecrawlService(currentSettings.llm.firecrawl);
                    const result = await service.testConnection();

                    if (result.success) {
                        button.setButtonText('Success!');
                    } else {
                        button.setButtonText('Failed');
                        console.error('[Doc Doctor] Firecrawl test failed:', result.message);
                    }
                    setTimeout(() => button.setButtonText('Test'), 2000);
                } catch (error) {
                    button.setButtonText('Error');
                    console.error('[Doc Doctor] Firecrawl test error:', error);
                    setTimeout(() => button.setButtonText('Test'), 2000);
                } finally {
                    button.setDisabled(false);
                }
            });
    });

    // Feature toggles
    containerEl.createEl('h4', { text: 'Features' });

    new Setting(containerEl)
        .setName('Web Search')
        .setDesc('Search the web for relevant sources when document refinement is low')
        .addToggle((toggle) => {
            toggle.setValue(firecrawlConfig.webSearchEnabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_FIRECRAWL_WEB_SEARCH',
                    payload: { enabled: value },
                });
            });
        });

    new Setting(containerEl)
        .setName('URL Scraping')
        .setDesc('Scrape URLs found in the document to include their content')
        .addToggle((toggle) => {
            toggle.setValue(firecrawlConfig.urlScrapingEnabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_FIRECRAWL_URL_SCRAPING',
                    payload: { enabled: value },
                });
            });
        });

    new Setting(containerEl)
        .setName('Smart Connections')
        .setDesc('Include related notes from your vault using Smart Connections plugin')
        .addToggle((toggle) => {
            toggle.setValue(firecrawlConfig.smartConnectionsEnabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_FIRECRAWL_SMART_CONNECTIONS',
                    payload: { enabled: value },
                });
            });
        });

    // Thresholds and limits
    containerEl.createEl('h4', { text: 'Limits' });

    new Setting(containerEl)
        .setName('Web Search Refinement Threshold')
        .setDesc('Only search web when document refinement is below this value (0-1)')
        .addSlider((slider) => {
            slider
                .setLimits(0, 1, 0.1)
                .setValue(firecrawlConfig.webSearchRefinementThreshold)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_FIRECRAWL_REFINEMENT_THRESHOLD',
                        payload: { threshold: value },
                    });
                });
        });

    new Setting(containerEl)
        .setName('Max Search Results')
        .setDesc('Maximum number of web search results to include')
        .addSlider((slider) => {
            slider
                .setLimits(1, 10, 1)
                .setValue(firecrawlConfig.maxSearchResults)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_FIRECRAWL_MAX_SEARCH_RESULTS',
                        payload: { max: value },
                    });
                });
        });

    new Setting(containerEl)
        .setName('Max URLs to Scrape')
        .setDesc('Maximum number of URLs to scrape from the document')
        .addSlider((slider) => {
            slider
                .setLimits(0, 10, 1)
                .setValue(firecrawlConfig.maxUrlsToScrape)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_FIRECRAWL_MAX_URLS',
                        payload: { max: value },
                    });
                });
        });

    new Setting(containerEl)
        .setName('Max Related Notes')
        .setDesc('Maximum number of related vault notes to include')
        .addSlider((slider) => {
            slider
                .setLimits(0, 20, 1)
                .setValue(firecrawlConfig.maxRelatedNotes)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'LLM_SET_FIRECRAWL_MAX_RELATED_NOTES',
                        payload: { max: value },
                    });
                });
        });

    // ==========================================================================
    // STUB CONFIG SCHEMA SECTION
    // ==========================================================================

    containerEl.createEl('h2', { text: 'Custom Stub Schema' });
    containerEl.createEl('p', {
        text: 'Use an external YAML or JSON file to define custom stub types for LLM analysis.',
        cls: 'setting-item-description',
    });

    // Enable custom schema
    new Setting(containerEl)
        .setName('Enable Custom Schema')
        .setDesc('Load stub type definitions from an external file')
        .addToggle((toggle) => {
            toggle.setValue(llmConfig.stubConfigSchemaEnabled || false).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'LLM_SET_STUB_CONFIG_SCHEMA_ENABLED',
                    payload: { enabled: value },
                });
                containerEl.empty();
                LLMSettings({ plugin, containerEl });
            });
        });

    if (llmConfig.stubConfigSchemaEnabled) {
        // Schema file path
        new Setting(containerEl)
            .setName('Schema File Path')
            .setDesc('Vault-relative path to your stub schema file (YAML or JSON)')
            .addText((text) => {
                text.inputEl.style.width = '300px';
                text.setPlaceholder('config/stubs-schema.yaml')
                    .setValue(llmConfig.stubConfigSchemaPath || '')
                    .onChange((value) => {
                        plugin.settings.dispatch({
                            type: 'LLM_SET_STUB_CONFIG_SCHEMA_PATH',
                            payload: { path: value.trim() },
                        });
                    });
            });

        // Schema mode
        new Setting(containerEl)
            .setName('Schema Mode')
            .setDesc('How to handle custom schema relative to built-in types')
            .addDropdown((dropdown) => {
                dropdown
                    .addOption('merge', 'Merge - Extend/override built-in types')
                    .addOption('replace', 'Replace - Completely replace built-in types')
                    .setValue(llmConfig.stubConfigSchemaMode || 'merge')
                    .onChange((value) => {
                        plugin.settings.dispatch({
                            type: 'LLM_SET_STUB_CONFIG_SCHEMA_MODE',
                            payload: { mode: value as 'replace' | 'merge' },
                        });
                    });
            });

        // Help text
        containerEl.createEl('p', {
            text: 'Schema file should contain stub type definitions with key, displayName, color, and description fields. See documentation for format.',
            cls: 'setting-item-description',
        });
    }
};
