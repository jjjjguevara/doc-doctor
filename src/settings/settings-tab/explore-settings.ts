/**
 * Explore Settings Component
 *
 * Settings UI for the Explore view and Smart Connections integration.
 */

import { Notice, Setting } from 'obsidian';
import type LabeledAnnotations from '../../main';

interface Props {
    plugin: LabeledAnnotations;
    containerEl: HTMLElement;
}

export const ExploreSettings = ({ plugin, containerEl }: Props) => {
    const settings = plugin.settings.getValue();
    const scSettings = settings.smartConnections;

    // Header
    containerEl.createEl('h2', { text: 'Explore & Smart Connections' });
    containerEl.createEl('p', {
        text: 'Configure semantic search and related notes discovery. Works best with the Smart Connections plugin installed.',
        cls: 'setting-item-description',
    });

    // ==========================================================================
    // STATUS SECTION
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Status' });

    // Smart Connections Status
    const statusSetting = new Setting(containerEl)
        .setName('Smart Connections Status')
        .setDesc('Check the connection status and embedding availability');

    // Status display element
    const statusDisplay = statusSetting.descEl.createEl('div', {
        cls: 'sc-status-display',
    });

    const updateStatusDisplay = () => {
        statusDisplay.empty();

        if (plugin.smartConnectionsService) {
            const status = plugin.smartConnectionsService.getStatus();

            const statusIcon = statusDisplay.createEl('span', {
                cls: `sc-status-icon ${status.smartConnections ? 'available' : 'unavailable'}`,
            });
            statusIcon.textContent = status.smartConnections ? '●' : '○';

            const statusText = statusDisplay.createEl('span', { cls: 'sc-status-text' });

            if (status.smartConnections) {
                statusText.textContent = status.embeddingsCount > 0
                    ? `Connected (${status.embeddingsCount} embeddings)`
                    : 'Connected';
            } else if (status.fallbackMode) {
                statusText.textContent = 'Fallback mode (keyword search)';
                if (status.error) {
                    const errorEl = statusDisplay.createEl('div', { cls: 'sc-status-error' });
                    errorEl.textContent = status.error;
                }
            } else {
                statusText.textContent = 'Not available';
            }
        } else {
            statusDisplay.createEl('span', { text: 'Service not initialized' });
        }
    };

    updateStatusDisplay();

    statusSetting.addButton((button) => {
        button.setButtonText('Refresh').onClick(() => {
            if (plugin.smartConnectionsService) {
                plugin.smartConnectionsService.refreshApiReference();
            }
            updateStatusDisplay();
            new Notice('Status refreshed');
        });
    });

    statusSetting.addExtraButton((button) => {
        button
            .setIcon('external-link')
            .setTooltip('Install Smart Connections')
            .onClick(() => {
                window.open('obsidian://show-plugin?id=smart-connections', '_blank');
            });
    });

    // Add CSS for status display
    const style = document.createElement('style');
    style.textContent = `
        .sc-status-display {
            display: flex;
            flex-wrap: wrap;
            align-items: center;
            gap: 8px;
            margin-top: 8px;
            padding: 8px 12px;
            background: var(--background-secondary);
            border-radius: 6px;
            font-size: var(--font-ui-small);
        }
        .sc-status-icon {
            font-size: 12px;
        }
        .sc-status-icon.available {
            color: var(--color-green);
        }
        .sc-status-icon.unavailable {
            color: var(--text-muted);
        }
        .sc-status-error {
            width: 100%;
            color: var(--text-warning);
            font-size: var(--font-ui-smaller);
            margin-top: 4px;
        }
    `;
    containerEl.appendChild(style);

    // ==========================================================================
    // FEATURE TOGGLES
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Features' });

    // Enable Smart Connections integration
    new Setting(containerEl)
        .setName('Enable Smart Connections')
        .setDesc('Use Smart Connections for semantic search when available')
        .addToggle((toggle) => {
            toggle.setValue(scSettings.enabled).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_SMART_CONNECTIONS_ENABLED',
                    payload: { enabled: value },
                });
                // Refresh the display
                containerEl.empty();
                ExploreSettings({ plugin, containerEl });
            });
        });

    if (!scSettings.enabled) {
        return;
    }

    // Auto-populate related property
    new Setting(containerEl)
        .setName('Auto-populate Related Property')
        .setDesc('Automatically suggest related notes to add to frontmatter')
        .addToggle((toggle) => {
            toggle.setValue(scSettings.autoPopulateRelated).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_SMART_CONNECTIONS_AUTO_POPULATE',
                    payload: { enabled: value },
                });
            });
        });

    // Related property name
    new Setting(containerEl)
        .setName('Related Property Name')
        .setDesc('Frontmatter property name for storing related notes (default: "related")')
        .addText((text) => {
            text.setPlaceholder('related')
                .setValue(scSettings.relatedPropertyName ?? 'related')
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'SET_SMART_CONNECTIONS_RELATED_PROPERTY_NAME',
                        payload: { propertyName: value || 'related' },
                    });
                });
        });

    // Warn on duplicates
    new Setting(containerEl)
        .setName('Warn on Duplicates')
        .setDesc('Show warning when similar notes are detected')
        .addToggle((toggle) => {
            toggle.setValue(scSettings.warnOnDuplicates).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_SMART_CONNECTIONS_WARN_DUPLICATES',
                    payload: { enabled: value },
                });
            });
        });

    // Suggest links
    new Setting(containerEl)
        .setName('Suggest Links')
        .setDesc('Show wikilink suggestions based on semantic similarity')
        .addToggle((toggle) => {
            toggle.setValue(scSettings.suggestLinks).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_SMART_CONNECTIONS_SUGGEST_LINKS',
                    payload: { enabled: value },
                });
            });
        });

    // ==========================================================================
    // THRESHOLDS AND LIMITS
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Thresholds & Limits' });

    // Related notes limit
    new Setting(containerEl)
        .setName('Related Notes Limit')
        .setDesc('Maximum number of related notes to display')
        .addSlider((slider) => {
            slider
                .setLimits(3, 20, 1)
                .setValue(scSettings.relatedNotesLimit)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'SET_SMART_CONNECTIONS_RELATED_LIMIT',
                        payload: { limit: value },
                    });
                });
        });

    // Related threshold
    new Setting(containerEl)
        .setName('Related Notes Threshold')
        .setDesc('Minimum similarity score to show as related. Smart Connections scores often range 0.2-0.6.')
        .addSlider((slider) => {
            slider
                .setLimits(0.1, 0.9, 0.05)
                .setValue(scSettings.relatedThreshold)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'SET_SMART_CONNECTIONS_RELATED_THRESHOLD',
                        payload: { threshold: value },
                    });
                });
        });

    // Duplicate threshold
    new Setting(containerEl)
        .setName('Duplicate Detection Threshold')
        .setDesc('Similarity threshold for duplicate warnings (0-1)')
        .addSlider((slider) => {
            slider
                .setLimits(0.7, 0.95, 0.05)
                .setValue(scSettings.duplicateThreshold)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'SET_SMART_CONNECTIONS_DUPLICATE_THRESHOLD',
                        payload: { threshold: value },
                    });
                });
        });

    // Link suggestion confidence
    new Setting(containerEl)
        .setName('Link Suggestion Confidence')
        .setDesc('Minimum confidence for link suggestions (0-1)')
        .addSlider((slider) => {
            slider
                .setLimits(0.4, 0.9, 0.05)
                .setValue(scSettings.linkSuggestionMinConfidence)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'SET_SMART_CONNECTIONS_LINK_CONFIDENCE',
                        payload: { confidence: value },
                    });
                });
        });

    // ==========================================================================
    // PERFORMANCE
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Performance' });

    // Cache results
    new Setting(containerEl)
        .setName('Cache Results')
        .setDesc('Cache search results to improve performance')
        .addToggle((toggle) => {
            toggle.setValue(scSettings.cacheResults).onChange((value) => {
                plugin.settings.dispatch({
                    type: 'SET_SMART_CONNECTIONS_CACHE_ENABLED',
                    payload: { enabled: value },
                });
            });
        });

    // Cache duration
    new Setting(containerEl)
        .setName('Cache Duration')
        .setDesc('How long to cache results (minutes)')
        .addSlider((slider) => {
            slider
                .setLimits(1, 30, 1)
                .setValue(scSettings.cacheDurationMinutes)
                .setDynamicTooltip()
                .onChange((value) => {
                    plugin.settings.dispatch({
                        type: 'SET_SMART_CONNECTIONS_CACHE_DURATION',
                        payload: { minutes: value },
                    });
                });
        });

    // Clear cache button
    new Setting(containerEl)
        .setName('Clear Cache')
        .setDesc('Clear the cached search results')
        .addButton((button) => {
            button.setButtonText('Clear Cache').onClick(() => {
                if (plugin.smartConnectionsService) {
                    plugin.smartConnectionsService.clearCache();
                    new Notice('Cache cleared');
                }
            });
        });

    // ==========================================================================
    // DIAGNOSTICS
    // ==========================================================================

    containerEl.createEl('h3', { text: 'Diagnostics' });

    // Test search
    new Setting(containerEl)
        .setName('Test Semantic Search')
        .setDesc('Run a test search to verify the integration')
        .addButton((button) => {
            button.setButtonText('Run Test').onClick(async () => {
                button.setButtonText('Testing...');
                button.setDisabled(true);

                try {
                    if (!plugin.smartConnectionsService) {
                        new Notice('SmartConnectionsService not initialized');
                        return;
                    }

                    const activeFile = plugin.app.workspace.getActiveFile();
                    if (!activeFile) {
                        new Notice('No active file - open a note first');
                        return;
                    }

                    const startTime = Date.now();
                    const results = await plugin.smartConnectionsService.findRelated(activeFile.path, 5);
                    const duration = Date.now() - startTime;

                    if (results.length > 0) {
                        const status = plugin.smartConnectionsService.getStatus();
                        const method = status.smartConnections ? 'embeddings' : 'keywords';
                        new Notice(`Found ${results.length} related notes via ${method} in ${duration}ms`);

                        console.group('[Doc Doctor] Test Results');
                        results.forEach((r, i) => {
                            console.log(`${i + 1}. ${r.title} (${Math.round(r.similarity * 100)}%) [${r.method}]`);
                        });
                        console.groupEnd();
                    } else {
                        new Notice('No related notes found');
                    }

                    button.setButtonText('Test Complete');
                    setTimeout(() => button.setButtonText('Run Test'), 2000);
                } catch (error) {
                    console.error('[Doc Doctor] Test error:', error);
                    new Notice('Test failed - check console for details');
                    button.setButtonText('Failed');
                    setTimeout(() => button.setButtonText('Run Test'), 2000);
                } finally {
                    button.setDisabled(false);
                }
            });
        });
};
