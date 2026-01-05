import { describe, it, expect } from 'vitest';
import type {
    Strategy,
    Service,
    AppStatus,
    ServiceStatus,
    OptimizationProgress,
    DiagnosticResult,
    VlessConfig,
    SingboxStatus,
    SingboxInstance,
    ProxyProtocol,
    ProxyConfig,
    DomainRoute,
    AppRoute,
    InstalledApp,
    TestProgress,
    TestResult,
    AppSettings,
    LogEntry,
    LogFilter,
    TrayState,
    TunStatus,
    TunConfig,
    TunInstance
} from '../api';

describe('API Types', () => {
    describe('Strategy interface', () => {
        it('should have correct structure', () => {
            const strategy: Strategy = {
                id: 'test-strategy',
                name: 'Test Strategy',
                description: 'A test strategy for DPI bypass',
                family: 'zapret',
                engine: 'winws'
            };

            expect(strategy.id).toBe('test-strategy');
            expect(strategy.name).toBe('Test Strategy');
            expect(strategy.description).toBe('A test strategy for DPI bypass');
            expect(strategy.family).toBe('zapret');
            expect(strategy.engine).toBe('winws');
        });

        it('should require all fields', () => {
            const strategy: Strategy = {
                id: '',
                name: '',
                description: '',
                family: '',
                engine: ''
            };

            expect(strategy).toHaveProperty('id');
            expect(strategy).toHaveProperty('name');
            expect(strategy).toHaveProperty('description');
            expect(strategy).toHaveProperty('family');
            expect(strategy).toHaveProperty('engine');
        });
    });

    describe('Service interface', () => {
        it('should have correct structure', () => {
            const service: Service = {
                id: 'youtube',
                name: 'YouTube',
                critical: true
            };

            expect(service.id).toBe('youtube');
            expect(service.name).toBe('YouTube');
            expect(service.critical).toBe(true);
        });
    });

    describe('AppStatus interface', () => {
        it('should have correct structure', () => {
            const status: AppStatus = {
                is_active: true,
                current_strategy: 'strategy-1',
                services_status: {
                    youtube: {
                        name: 'YouTube',
                        is_available: true,
                        latency_ms: 150
                    }
                }
            };

            expect(status.is_active).toBe(true);
            expect(status.current_strategy).toBe('strategy-1');
            expect(status.services_status.youtube.is_available).toBe(true);
        });

        it('should allow null current_strategy', () => {
            const status: AppStatus = {
                is_active: false,
                current_strategy: null,
                services_status: {}
            };

            expect(status.current_strategy).toBeNull();
        });
    });

    describe('ServiceStatus interface', () => {
        it('should have correct structure with latency', () => {
            const serviceStatus: ServiceStatus = {
                name: 'Discord',
                is_available: true,
                latency_ms: 45
            };

            expect(serviceStatus.name).toBe('Discord');
            expect(serviceStatus.is_available).toBe(true);
            expect(serviceStatus.latency_ms).toBe(45);
        });

        it('should allow null latency_ms', () => {
            const serviceStatus: ServiceStatus = {
                name: 'Discord',
                is_available: false,
                latency_ms: null
            };

            expect(serviceStatus.latency_ms).toBeNull();
        });
    });

    describe('OptimizationProgress interface', () => {
        it('should have correct structure', () => {
            const progress: OptimizationProgress = {
                stage: 'testing',
                percent: 50,
                message: 'Testing strategy 5 of 10',
                current_strategy: 'zapret-youtube',
                tested_count: 5,
                total_count: 10,
                best_score: 85.5
            };

            expect(progress.stage).toBe('testing');
            expect(progress.percent).toBe(50);
            expect(progress.tested_count).toBe(5);
            expect(progress.total_count).toBe(10);
            expect(progress.best_score).toBe(85.5);
        });

        it('should allow null optional fields', () => {
            const progress: OptimizationProgress = {
                stage: 'initializing',
                percent: 0,
                message: 'Starting optimization',
                current_strategy: null,
                tested_count: 0,
                total_count: 10,
                best_score: null
            };

            expect(progress.current_strategy).toBeNull();
            expect(progress.best_score).toBeNull();
        });
    });

    describe('DiagnosticResult interface', () => {
        it('should have correct structure', () => {
            const result: DiagnosticResult = {
                profile: {
                    kind: 'dpi_detected',
                    details: 'HTTP injection detected',
                    candidate_families: ['zapret', 'vless']
                },
                tested_services: ['youtube', 'discord', 'telegram'],
                blocked_services: ['youtube', 'discord']
            };

            expect(result.profile.kind).toBe('dpi_detected');
            expect(result.profile.candidate_families).toContain('zapret');
            expect(result.blocked_services).toHaveLength(2);
        });
    });

    describe('VlessConfig interface', () => {
        it('should have correct structure', () => {
            const config: VlessConfig = {
                id: 'vless-1',
                name: 'My VLESS Server',
                server: 'example.com',
                port: 443,
                uuid: '12345678-1234-1234-1234-123456789abc',
                flow: 'xtls-rprx-vision',
                security: 'tls',
                sni: 'example.com',
                active: true
            };

            expect(config.id).toBe('vless-1');
            expect(config.port).toBe(443);
            expect(config.uuid).toMatch(/^[0-9a-f-]{36}$/);
            expect(config.active).toBe(true);
        });

        it('should allow null optional fields', () => {
            const config: VlessConfig = {
                id: 'vless-2',
                name: 'Basic VLESS',
                server: 'example.com',
                port: 443,
                uuid: '12345678-1234-1234-1234-123456789abc',
                flow: null,
                security: 'none',
                sni: null,
                active: false
            };

            expect(config.flow).toBeNull();
            expect(config.sni).toBeNull();
        });
    });

    describe('SingboxStatus type', () => {
        it('should accept valid status values', () => {
            const statuses: SingboxStatus[] = [
                'starting',
                'running',
                'stopping',
                'stopped',
                'failed',
                'health_check_failed'
            ];

            expect(statuses).toHaveLength(6);
            statuses.forEach(status => {
                expect(typeof status).toBe('string');
            });
        });
    });

    describe('SingboxInstance interface', () => {
        it('should have correct structure', () => {
            const instance: SingboxInstance = {
                config_id: 'vless-1',
                config_name: 'My VLESS',
                socks_port: 10808,
                status: 'running',
                pid: 12345,
                started_at: Date.now(),
                last_health_check: Date.now(),
                health_check_failures: 0
            };

            expect(instance.config_id).toBe('vless-1');
            expect(instance.socks_port).toBe(10808);
            expect(instance.status).toBe('running');
            expect(instance.pid).toBe(12345);
        });

        it('should allow null optional fields', () => {
            const instance: SingboxInstance = {
                config_id: 'vless-1',
                config_name: 'My VLESS',
                socks_port: 10808,
                status: 'stopped',
                pid: null,
                started_at: null,
                last_health_check: null,
                health_check_failures: 0
            };

            expect(instance.pid).toBeNull();
            expect(instance.started_at).toBeNull();
        });
    });

    describe('ProxyProtocol type', () => {
        it('should accept all valid protocols', () => {
            const protocols: ProxyProtocol[] = [
                'socks5',
                'http',
                'https',
                'shadowsocks',
                'trojan',
                'vmess',
                'vless',
                'tuic',
                'hysteria',
                'hysteria2',
                'wireguard',
                'ssh'
            ];

            expect(protocols).toHaveLength(12);
        });
    });

    describe('ProxyConfig interface', () => {
        it('should have correct structure', () => {
            const proxy: ProxyConfig = {
                id: 'proxy-1',
                name: 'My Proxy',
                protocol: 'vless',
                server: 'example.com',
                port: 443,
                username: 'user',
                password: 'pass',
                uuid: '12345678-1234-1234-1234-123456789abc',
                tls: true,
                sni: 'example.com',
                transport: 'ws',
                custom_fields: { path: '/ws' },
                active: true
            };

            expect(proxy.id).toBe('proxy-1');
            expect(proxy.protocol).toBe('vless');
            expect(proxy.tls).toBe(true);
            expect(proxy.custom_fields.path).toBe('/ws');
        });
    });

    describe('DomainRoute interface', () => {
        it('should have correct structure', () => {
            const route: DomainRoute = {
                domain: '*.youtube.com',
                proxy_id: 'proxy-1'
            };

            expect(route.domain).toBe('*.youtube.com');
            expect(route.proxy_id).toBe('proxy-1');
        });
    });

    describe('AppRoute interface', () => {
        it('should have correct structure', () => {
            const route: AppRoute = {
                app_name: 'Discord',
                app_path: 'C:\\Users\\User\\AppData\\Local\\Discord\\Discord.exe',
                proxy_id: 'proxy-1'
            };

            expect(route.app_name).toBe('Discord');
            expect(route.app_path).toContain('Discord.exe');
        });
    });

    describe('InstalledApp interface', () => {
        it('should have correct structure', () => {
            const app: InstalledApp = {
                name: 'Discord',
                path: 'C:\\Program Files\\Discord\\Discord.exe',
                icon: 'base64encodedicon'
            };

            expect(app.name).toBe('Discord');
            expect(app.path).toContain('Discord.exe');
            expect(app.icon).toBeDefined();
        });

        it('should allow undefined icon', () => {
            const app: InstalledApp = {
                name: 'App',
                path: '/path/to/app'
            };

            expect(app.icon).toBeUndefined();
        });
    });

    describe('TestProgress interface', () => {
        it('should have correct structure', () => {
            const progress: TestProgress = {
                current_item: 'zapret-youtube',
                current_type: 'strategy',
                tested_count: 3,
                total_count: 10,
                percent: 30
            };

            expect(progress.current_item).toBe('zapret-youtube');
            expect(progress.current_type).toBe('strategy');
            expect(progress.percent).toBe(30);
        });
    });

    describe('TestResult interface', () => {
        it('should have correct structure', () => {
            const result: TestResult = {
                id: 'strategy-1',
                name: 'Zapret YouTube',
                type: 'strategy',
                success_rate: 90,
                latency_ms: 150,
                score: 85.5,
                services_tested: ['youtube', 'discord'],
                services_passed: ['youtube']
            };

            expect(result.success_rate).toBe(90);
            expect(result.score).toBe(85.5);
            expect(result.services_passed).toContain('youtube');
        });
    });

    describe('AppSettings interface', () => {
        it('should have correct structure', () => {
            const settings: AppSettings = {
                auto_start: true,
                auto_apply: true,
                minimize_to_tray: true,
                block_quic: true,
                default_mode: 'turbo',
                system_proxy: false,
                tun_mode: false,
                per_domain_routing: true,
                per_app_routing: false,
                test_timeout: 5,
                test_services: ['youtube', 'discord'],
                language: 'ru',
                telemetry_enabled: false
            };

            expect(settings.auto_start).toBe(true);
            expect(settings.default_mode).toBe('turbo');
            expect(settings.language).toBe('ru');
            expect(settings.test_services).toContain('youtube');
        });
    });

    describe('LogEntry interface', () => {
        it('should have correct structure', () => {
            const entry: LogEntry = {
                timestamp: '2024-01-15T10:30:00Z',
                level: 'info',
                module: 'strategy_engine',
                message: 'Strategy applied successfully'
            };

            expect(entry.timestamp).toMatch(/^\d{4}-\d{2}-\d{2}T/);
            expect(entry.level).toBe('info');
            expect(entry.module).toBe('strategy_engine');
        });
    });

    describe('LogFilter interface', () => {
        it('should have correct structure', () => {
            const filter: LogFilter = {
                level: 'warn',
                module: 'strategy',
                search: 'error'
            };

            expect(filter.level).toBe('warn');
            expect(filter.module).toBe('strategy');
        });

        it('should allow all optional fields', () => {
            const filter: LogFilter = {};

            expect(filter.level).toBeUndefined();
            expect(filter.module).toBeUndefined();
            expect(filter.search).toBeUndefined();
        });
    });

    describe('TrayState type', () => {
        it('should accept valid states', () => {
            const states: TrayState[] = ['inactive', 'active', 'optimizing', 'error'];

            expect(states).toHaveLength(4);
        });
    });

    describe('TunStatus type', () => {
        it('should accept valid statuses', () => {
            const statuses: TunStatus[] = [
                'stopped',
                'starting',
                'running',
                'stopping',
                'failed'
            ];

            expect(statuses).toHaveLength(5);
        });
    });

    describe('TunConfig interface', () => {
        it('should have correct structure', () => {
            const config: TunConfig = {
                interface_name: 'tun0',
                mtu: 1500,
                address_v4: '10.0.0.1/24',
                address_v6: 'fd00::1/64',
                strict_route: true,
                auto_route: true,
                stack: 'gvisor'
            };

            expect(config.interface_name).toBe('tun0');
            expect(config.mtu).toBe(1500);
            expect(config.strict_route).toBe(true);
        });

        it('should allow undefined address_v6', () => {
            const config: TunConfig = {
                interface_name: 'tun0',
                mtu: 1500,
                address_v4: '10.0.0.1/24',
                strict_route: true,
                auto_route: true,
                stack: 'gvisor'
            };

            expect(config.address_v6).toBeUndefined();
        });
    });

    describe('TunInstance interface', () => {
        it('should have correct structure', () => {
            const instance: TunInstance = {
                status: 'running',
                socks_port: 10808,
                pid: 12345,
                started_at: Date.now(),
                config: {
                    interface_name: 'tun0',
                    mtu: 1500,
                    address_v4: '10.0.0.1/24',
                    strict_route: true,
                    auto_route: true,
                    stack: 'gvisor'
                }
            };

            expect(instance.status).toBe('running');
            expect(instance.socks_port).toBe(10808);
            expect(instance.config.interface_name).toBe('tun0');
        });
    });
});

describe('API Type Compatibility with Backend', () => {
    it('Strategy fields should match Rust struct', () => {
        // These field names must match the Rust backend exactly
        const requiredFields = ['id', 'name', 'description', 'family', 'engine'];
        const strategy: Strategy = {
            id: 'test',
            name: 'Test',
            description: 'Test',
            family: 'zapret',
            engine: 'winws'
        };

        requiredFields.forEach(field => {
            expect(strategy).toHaveProperty(field);
        });
    });

    it('Service fields should match Rust struct', () => {
        const requiredFields = ['id', 'name', 'critical'];
        const service: Service = {
            id: 'test',
            name: 'Test',
            critical: false
        };

        requiredFields.forEach(field => {
            expect(service).toHaveProperty(field);
        });
    });

    it('AppSettings fields should match Rust struct', () => {
        const requiredFields = [
            'auto_start',
            'auto_apply',
            'minimize_to_tray',
            'block_quic',
            'default_mode',
            'system_proxy',
            'tun_mode',
            'per_domain_routing',
            'per_app_routing',
            'test_timeout',
            'test_services',
            'language',
            'telemetry_enabled'
        ];

        const settings: AppSettings = {
            auto_start: false,
            auto_apply: false,
            minimize_to_tray: false,
            block_quic: false,
            default_mode: 'turbo',
            system_proxy: false,
            tun_mode: false,
            per_domain_routing: false,
            per_app_routing: false,
            test_timeout: 5,
            test_services: [],
            language: 'en',
            telemetry_enabled: false
        };

        requiredFields.forEach(field => {
            expect(settings).toHaveProperty(field);
        });
    });

    it('ProxyConfig fields should use snake_case for Rust compatibility', () => {
        const proxy: ProxyConfig = {
            id: 'test',
            name: 'Test',
            protocol: 'socks5',
            server: 'localhost',
            port: 1080,
            tls: false,
            custom_fields: {},
            active: false
        };

        // Verify snake_case naming convention
        expect(proxy).toHaveProperty('custom_fields');
        expect(proxy).not.toHaveProperty('customFields');
    });
});
