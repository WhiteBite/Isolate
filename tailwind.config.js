/** @type {import('tailwindcss').Config} */
export default {
    darkMode: 'class',
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        extend: {
            colors: {
                // AAA Premium Dark Theme (Linear/Arc/Vercel style)
                void: {
                    DEFAULT: '#09090b',
                    50: '#18181b',
                    100: '#27272a',
                    200: '#3f3f46',
                    300: '#52525b',
                },
                glass: {
                    border: 'rgba(255, 255, 255, 0.05)',
                    'border-active': 'rgba(255, 255, 255, 0.1)',
                    surface: 'rgba(24, 24, 27, 0.4)',
                },
                electric: {
                    DEFAULT: '#6366F1',
                    glow: 'rgba(99, 102, 241, 0.5)',
                    dim: 'rgba(99, 102, 241, 0.15)',
                },
                neon: {
                    green: '#10B981',
                    red: '#F87171',
                    yellow: '#FBBF24',
                    cyan: '#22D3EE',
                    purple: '#A78BFA',
                },
                text: {
                    primary: '#F4F4F5',
                    secondary: '#A1A1AA',
                    muted: '#A1A1AA',
                    placeholder: '#A1A1AA',
                },
            },
            fontFamily: {
                sans: ['Inter', 'Geist', 'system-ui', 'sans-serif'],
                mono: ['JetBrains Mono', 'Geist Mono', 'monospace'],
            },
            boxShadow: {
                'glow': '0 0 40px -10px rgba(99, 102, 241, 0.5)',
                'glow-lg': '0 0 60px -15px rgba(99, 102, 241, 0.6)',
                'glow-green': '0 0 40px -10px rgba(16, 185, 129, 0.5)',
                'glow-green-lg': '0 0 60px -10px rgba(16, 185, 129, 0.6)',
                'glow-red': '0 0 40px -10px rgba(248, 113, 113, 0.5)',
                'glow-cyan': '0 0 40px -10px rgba(34, 211, 238, 0.5)',
                'glow-indigo': '0 0 50px -10px rgba(79, 70, 229, 0.5)',
                'inner-dark': 'inset 0 2px 4px 0 rgba(0, 0, 0, 0.6)',
                'inner-light': 'inset 0 1px 0 0 rgba(255, 255, 255, 0.05)',
                'card': '0 1px 3px 0 rgba(0, 0, 0, 0.3), 0 1px 2px -1px rgba(0, 0, 0, 0.3)',
            },
            backgroundImage: {
                'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
                'gradient-dark': 'linear-gradient(to bottom right, #09090b, #000000)',
                'gradient-card': 'linear-gradient(to bottom right, rgba(39, 39, 42, 0.5), rgba(24, 24, 27, 0.3))',
                'gradient-ambient': 'radial-gradient(ellipse at top, rgba(99, 102, 241, 0.15), transparent 50%)',
            },
            animation: {
                'pulse-glow': 'pulse-glow 3s ease-in-out infinite',
                'pulse-slow': 'pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite',
                'slide-up': 'slide-up 0.3s ease-out',
                'slide-down': 'slide-down 0.3s ease-out',
                'fade-in': 'fade-in 0.2s ease-out',
                'fade-slide-up': 'fade-slide-up 0.4s ease-out',
                'scanning-bar': 'scanning-bar 1.5s ease-in-out infinite',
            },
            keyframes: {
                'scanning-bar': {
                    '0%': { width: '0%', marginLeft: '0%' },
                    '50%': { width: '30%', marginLeft: '35%' },
                    '100%': { width: '0%', marginLeft: '100%' },
                },
                'pulse-glow': {
                    '0%, 100%': { boxShadow: '0 0 40px -10px rgba(99, 102, 241, 0.4)' },
                    '50%': { boxShadow: '0 0 60px -10px rgba(99, 102, 241, 0.7)' },
                },
                'slide-up': {
                    '0%': { transform: 'translateY(10px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                'slide-down': {
                    '0%': { transform: 'translateY(-10px)', opacity: '0' },
                    '100%': { transform: 'translateY(0)', opacity: '1' },
                },
                'fade-in': {
                    '0%': { opacity: '0' },
                    '100%': { opacity: '1' },
                },
                'fade-slide-up': {
                    '0%': { opacity: '0', transform: 'translateY(10px)' },
                    '100%': { opacity: '1', transform: 'translateY(0)' },
                },
            },
        }
    },
    plugins: []
};
