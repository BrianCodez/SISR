<script lang="ts">
import type { components } from '$lib/api/openapi';
import FloatingCard from '../FloatingCard.svelte';
import { onMount, untrack } from 'svelte';

import IcoClose from '~icons/mdi/close';
import IcoGamepad from '~icons/fluent/xbox-controller-16-filled';
import IcoSettings from '~icons/mdi/cog';
import { client, wrapClientError } from '$lib/api/client';
import { log } from '$lib/log';
import { toast } from '$lib/toaster/toaster.svelte';

const STEAM_DESKTOP_CONFIG_APPID = 413080;
type SteamShortcutProfile = components['schemas']['SteamShortcutProfile'];

const {
	steamStatusInfo,
	inputInfo,
	onClose
}: {
	steamStatusInfo: components['schemas']['SteamStatus'];
	inputInfo: components['schemas']['InputInfoResponse'];
	onClose?: () => void;
} = $props();

let allowDesktopConfig = $state(untrack(() => !inputInfo.force_controller_config));
let steamOverlayEnabled = $state(untrack(() => inputInfo.fullscreen));
let firstControl: HTMLInputElement | undefined;
let steamInputProfiles = $state<SteamShortcutProfile[]>([]);
let selectedSteamInputProfile = $state<number | undefined>(
	untrack(() => inputInfo.force_controller_config_app_id ?? steamStatusInfo.marker_app_id ?? undefined)
);
let profileLoadError = $state<string | undefined>();
let loadingProfiles = $state(false);
let armedSteamInputProfile = $state<number | undefined>();

onMount(() => {
	loadSteamInputProfiles();
	firstControl?.focus();
});

const reportActionFailure = (message: string, e: unknown) => {
	log.error(message, 'error', e);
	toast({
		color: 'firebrick',
		message
	});
};

const setAllowDesktopConfig = (allow: boolean) => {
	allowDesktopConfig = allow;
	void client
		.POST('/api/v1/force_controller_config', {
			body: {
				enforce: !allow
			}
		})
		.catch((e) => {
			allowDesktopConfig = !allow;
			reportActionFailure('Failed to change allow desktop config', e);
		});
};

const setSteamOverlayEnabled = (enabled: boolean) => {
	steamOverlayEnabled = enabled;
	void client
		.POST('/api/v1/steam_overlay', {
			body: {
				enabled
			}
		})
		.catch((e) => {
			steamOverlayEnabled = !enabled;
			reportActionFailure('Failed to change Steam Overlay setting', e);
		});
};

const openSteamInputConfigurator = () => {
	void client
		.POST('/api/v1/open_steam_controller_config', {
			body: {
				app_id: allowDesktopConfig
					? STEAM_DESKTOP_CONFIG_APPID
					: selectedSteamInputProfile || steamStatusInfo.marker_app_id
			}
		})
		.catch((e) => reportActionFailure('Failed to open Steam Input Layout configurator', e));
};

const loadSteamInputProfiles = () => {
	if (steamStatusInfo.no_steam_mode) {
		return;
	}

	loadingProfiles = true;
	void client
		.GET('/api/v1/steam_shortcuts')
		.then(({ data, error }) => {
			if (error) {
				throw error;
			}
			steamInputProfiles = data?.shortcuts ?? [];
			selectedSteamInputProfile = data?.selected_app_id ?? selectedSteamInputProfile;
			profileLoadError = undefined;
		})
		.catch((e) => {
			profileLoadError = 'Steam shortcuts unavailable';
			log.warn('Failed to load Steam shortcuts', 'error', e);
		})
		.finally(() => {
			loadingProfiles = false;
		});
};

const launchSteamInputProfile = (profile: SteamShortcutProfile) => {
	if (armedSteamInputProfile !== profile.app_id) {
		armedSteamInputProfile = profile.app_id;
		return;
	}

	selectedSteamInputProfile = profile.app_id;
	void client
		.POST('/api/v1/steam_shortcuts/launch', {
			body: {
				app_id: profile.app_id
			}
		})
		.catch((e) => reportActionFailure('Failed to launch Steam Input profile', e));
};

const hideUi = () => {
	void client
		.POST('/api/v1/show_hide_ui', {
			body: {
				show: false
			}
		})
		.catch((e) => reportActionFailure('Failed to hide SISR UI', e));
};

const quitSisr = () => {
	if (!window.confirm('Quit SISR completely?')) {
		return;
	}

	void wrapClientError(client.POST('/api/v1/shutdown')).catch((e) =>
		reportActionFailure('Failed to quit SISR', e)
	);
};
</script>

<FloatingCard>
	<div id="card-content">
		<div>
			<IcoSettings style="width: 1.6em; height: 1.6em;" />
			<h2>Quick Settings</h2>
			<button class="plain" onclick={() => onClose?.()}>
				<IcoClose style="width: 1.6em; height: 1.6em;" />
			</button>
		</div>
		<div class="actions">
			<div class="checkbox-wrap">
				<label for="steam-overlay-enabled">Enable Steam Overlay</label>
				<input
					type="checkbox"
					id="steam-overlay-enabled"
					name="steam-overlay-enabled"
					bind:this={firstControl}
					onchange={(event) =>
						setSteamOverlayEnabled((event.currentTarget as HTMLInputElement).checked)}
					bind:checked={steamOverlayEnabled} />
			</div>
			{#if !steamStatusInfo.no_steam_mode}
				<div class="checkbox-wrap">
					<label for="allow-desktop-config">Allow Steam Input Desktop Layout</label>
					<input
						type="checkbox"
						id="allow-desktop-config"
						name="allow-desktop-config"
						onchange={(event) =>
							setAllowDesktopConfig((event.currentTarget as HTMLInputElement).checked)}
						bind:checked={allowDesktopConfig} />
				</div>
				<div class="profile-picker">
					<h3>Steam Input Profiles</h3>
					{#if loadingProfiles}
						<p>Loading Steam shortcuts...</p>
					{:else if steamInputProfiles.length === 0}
						<p>{profileLoadError ?? 'No Steam shortcuts found'}</p>
					{:else}
						<div class="profile-grid">
							{#each steamInputProfiles as profile (profile.app_id)}
								<button
									class:selected={profile.app_id === selectedSteamInputProfile ||
										profile.selected}
									class:armed={armedSteamInputProfile === profile.app_id}
									class="action-button profile-button"
									onclick={() => launchSteamInputProfile(profile)}>
									<span>{profile.name || profile.exe}</span>
									<small>
										{armedSteamInputProfile === profile.app_id
											? 'Press A again to launch'
											: `Steam shortcut ${profile.app_id}`}
									</small>
								</button>
							{/each}
						</div>
					{/if}
					{#if profileLoadError && steamInputProfiles.length > 0}
						<span>{profileLoadError}</span>
					{/if}
				</div>
				<button class="action-button" onclick={openSteamInputConfigurator}>
					<IcoGamepad style="width: 1.4em; height: 1.4em;" />
					<div>
						<span>Open Steam Input Layout configurator</span>
						{#if allowDesktopConfig}
							<span>(Desktop Layout)</span>
						{/if}
					</div>
				</button>
			{/if}
			<button class="action-button" onclick={hideUi}>
				<IcoClose style="width: 1.4em; height: 1.4em;" />
				<span>Hide UI</span>
			</button>
			<button class="action-button danger" onclick={quitSisr}>Quit SISR</button>
		</div>
	</div>
</FloatingCard>

<style lang="postcss">
#card-content {
	display: flex;
	flex-direction: column;
	gap: 1em;
	height: 100%;

	& > :first-child {
		display: grid;
		grid-template-columns: min-content 1fr min-content;
		justify-content: center;
		align-items: center;
		gap: 1em;
		& button {
			padding: 0.5em;
		}
	}
	& > :last-child {
		overflow: auto;
		display: grid;
		gap: 1em;
		padding: 1em;
		width: 100%;
	}
}

.checkbox-wrap {
	display: grid;
	grid-template-columns: 1fr min-content;
	gap: 1em;
	align-items: center;
	justify-content: stretch;
	border-radius: 0.5em;
	padding: 0.75em 1em;
	background: rgba(255, 255, 255, 0.06);

	& input {
		width: 1.5em;
		height: 1.5em;
	}
}

.profile-picker {
	display: grid;
	gap: 0.5em;
	border-radius: 0.5em;
	padding: 0.75em 1em;
	background: rgba(255, 255, 255, 0.06);

	& p {
		margin: 0;
	}

	& > span {
		color: firebrick;
		font-size: 0.9em;
	}
}

.profile-grid {
	display: grid;
	gap: 0.5em;
	max-height: 14em;
	overflow: auto;
}

.profile-button {
	grid-auto-flow: row;
	gap: 0.25em;

	&.selected,
	&.armed {
		outline: 0.2em solid var(--color-accent, white);
	}

	&.armed {
		background: color-mix(in srgb, firebrick 45%, transparent);
	}

	& small {
		opacity: 0.7;
	}
}

button {
	display: grid;
	grid-auto-flow: column;
	gap: 0.5em;
	place-items: center;
	width: fit-content;

	& div {
		display: grid;
		place-items: center;
		gap: 0.25em;
	}
}

.action-button {
	justify-content: center;
	min-height: 3.5em;
	width: 100%;
	padding: 0.75em 1em;
	text-align: center;
}

.danger {
	border-color: firebrick;
	color: white;
	background: color-mix(in srgb, firebrick 70%, transparent);
}
</style>
