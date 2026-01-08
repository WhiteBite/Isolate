<script lang="ts">
	/**
	 * StatusIndicator — универсальный компонент индикатора состояния
	 * Используется в Dashboard, Library, Network для отображения статуса
	 */

	type Status = 'idle' | 'loading' | 'active' | 'error' | 'recovering' | 'warning';
	type Size = 'sm' | 'md' | 'lg';

	interface Props {
		status: Status;
		size?: Size;
		label?: string;
		pulse?: boolean;
	}

	let { status, size = 'md', label, pulse }: Props = $props();

	// Размеры индикатора
	const sizeClasses: Record<Size, string> = {
		sm: 'w-2 h-2',
		md: 'w-3 h-3',
		lg: 'w-4 h-4'
	};

	// Цвета для каждого статуса
	const statusColors: Record<Status, string> = {
		idle: 'bg-gray-400',
		loading: 'bg-blue-500',
		active: 'bg-green-500',
		error: 'bg-red-500',
		recovering: 'bg-yellow-500',
		warning: 'bg-orange-500'
	};

	// Определяем нужна ли пульсация по умолчанию
	let shouldPulse = $derived(
		pulse !== undefined ? pulse : status === 'loading' || status === 'active' || status === 'recovering'
	);

	// Определяем нужна ли анимация spin (только для loading)
	let shouldSpin = $derived(status === 'loading');

	// Собираем классы для индикатора
	let indicatorClasses = $derived(
		[
			'rounded-full',
			'inline-block',
			'flex-shrink-0',
			sizeClasses[size],
			statusColors[status],
			shouldPulse && !shouldSpin ? 'animate-pulse' : '',
			shouldSpin ? 'animate-spin' : ''
		]
			.filter(Boolean)
			.join(' ')
	);

	// Классы для контейнера с label
	let containerClasses = $derived(label ? 'inline-flex items-center gap-2' : 'inline-block');

	// Размер текста в зависимости от размера индикатора
	const labelSizeClasses: Record<Size, string> = {
		sm: 'text-xs',
		md: 'text-sm',
		lg: 'text-base'
	};
</script>

<span class={containerClasses}>
	{#if shouldSpin}
		<!-- Для loading используем кольцо с анимацией spin -->
		<span
			class="{sizeClasses[size]} rounded-full border-2 border-blue-500 border-t-transparent animate-spin inline-block flex-shrink-0"
		></span>
	{:else}
		<span class={indicatorClasses}></span>
	{/if}

	{#if label}
		<span class="{labelSizeClasses[size]} text-gray-300">{label}</span>
	{/if}
</span>
