import { useLang, useI18n } from '@rspress/core/runtime';
import styles from './index.module.css';

const ArrowIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
    <path
      d="M3 8h10M9 4l4 4-4 4"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

const GitHubIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
  </svg>
);

const FeatureIcons = {
  viewport: () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
    </svg>
  ),
  rust: () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <circle cx="12" cy="12" r="10" />
      <path d="M12 8v4l3 3" />
    </svg>
  ),
  renderer: () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <rect x="3" y="3" width="7" height="7" />
      <rect x="14" y="3" width="7" height="7" />
      <rect x="3" y="14" width="7" height="7" />
      <rect x="14" y="14" width="7" height="7" />
    </svg>
  ),
  leptos: () => (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
    </svg>
  ),
};

const PIPELINE_STEPS = [
  { name: 'GridState', desc: 'model · viewport · selection' },
  { name: 'SceneBuilder', desc: 'rs-grid-scene' },
  { name: 'SceneFrame', desc: 'primitives' },
  { name: 'CanvasRenderer', desc: 'rs-grid-render-canvas' },
  { name: '<canvas>', desc: 'browser', isOutput: true },
];

const CRATES = ['core', 'scene', 'render', 'web', 'leptos'] as const;
const CRATE_NAMES: Record<(typeof CRATES)[number], string> = {
  core: 'rs-grid-core',
  scene: 'rs-grid-scene',
  render: 'rs-grid-render-canvas',
  web: 'rs-grid-web',
  leptos: 'rs-grid-leptos',
};

export default function HomeLayout() {
  const lang = useLang();
  const t = useI18n();
  const docsPath = lang === 'fr' ? '/fr/getting-started' : '/getting-started';

  return (
    <div className={styles.homeWrapper}>
      {/* Hero */}
      <section className={styles.hero}>
        <video
          className={styles.heroBgVideo}
          autoPlay
          loop
          muted
          playsInline
        >
          <source src="/rsgrid4k.mp4" type="video/mp4" />
        </video>
        <div className={styles.heroGlow} />
        <div className={styles.container}>
          <div className={styles.heroBadge}>
            <span className={styles.badgeDot} />
            {t('hero.badge')}
          </div>

          <h1 className={styles.heroTitle}>
            {t('hero.title1')}
            <br />
            <span className={styles.gradientText}>{t('hero.title2')}</span>
          </h1>

          <p className={styles.heroSub}>{t('hero.sub')}</p>

          <div className={styles.heroActions}>
            <a href={docsPath} className={styles.btnPrimary}>
              {t('hero.cta')}
              <ArrowIcon />
            </a>
            <a
              href="https://github.com/bpodwinski/rs-grid"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.btnOutline}
            >
              <GitHubIcon />
              {t('hero.github')}
            </a>
          </div>

          <div className={styles.heroStats}>
            <div className={styles.stat}>
              <span className={styles.statValue}>10M+</span>
              <span className={styles.statLabel}>{t('hero.stat.rows')}</span>
            </div>
            <div className={styles.statDivider} />
            <div className={styles.stat}>
              <span className={styles.statValue}>60fps</span>
              <span className={styles.statLabel}>{t('hero.stat.fps')}</span>
            </div>
            <div className={styles.statDivider} />
            <div className={styles.stat}>
              <span className={styles.statValue}>O(log n)</span>
              <span className={styles.statLabel}>{t('hero.stat.hit')}</span>
            </div>
            <div className={styles.statDivider} />
            <div className={styles.stat}>
              <span className={styles.statValue}>5</span>
              <span className={styles.statLabel}>{t('hero.stat.crates')}</span>
            </div>
          </div>
        </div>
      </section>

      {/* Features */}
      <section className={styles.section}>
        <div className={styles.container}>
          <div className={styles.sectionHeader}>
            <span className={styles.sectionTag}>{t('features.tag')}</span>
            <h2 className={styles.sectionTitle}>{t('features.title')}</h2>
            <p className={styles.sectionSub}>{t('features.sub')}</p>
          </div>

          <div className={styles.featuresGrid}>
            {(['viewport', 'rust', 'renderer', 'leptos'] as const).map(
              (key) => {
                const Icon = FeatureIcons[key];
                return (
                  <div key={key} className={styles.featureCard}>
                    <div className={styles.featureIcon}>
                      <Icon />
                    </div>
                    <h3 className={styles.featureCardTitle}>
                      {t(`features.${key}.title`)}
                    </h3>
                    <p className={styles.featureCardDesc}>
                      {t(`features.${key}.desc`)}
                    </p>
                  </div>
                );
              },
            )}
          </div>
        </div>
      </section>

      {/* Architecture */}
      <section className={`${styles.section} ${styles.sectionAlt}`}>
        <div className={styles.container}>
          <div className={styles.sectionHeader}>
            <span className={styles.sectionTag}>{t('arch.tag')}</span>
            <h2 className={styles.sectionTitle}>{t('arch.title')}</h2>
            <p className={styles.sectionSub}>{t('arch.sub')}</p>
          </div>

          <div className={styles.archPipeline}>
            {PIPELINE_STEPS.map((step, i) => (
              <>
                {i > 0 && (
                  <div key={`arrow-${i}`} className={styles.archArrow}>
                    &rarr;
                  </div>
                )}
                <div
                  key={step.name}
                  className={`${styles.archStep} ${step.isOutput ? styles.archStepOutput : ''}`}
                >
                  <div className={styles.archStepName}>{step.name}</div>
                  <div className={styles.archStepDesc}>{step.desc}</div>
                </div>
              </>
            ))}
          </div>

          <div className={styles.cratesGrid}>
            {CRATES.map((key) => (
              <div key={key} className={styles.crateCard}>
                <code className={styles.crateCardCode}>
                  {CRATE_NAMES[key]}
                </code>
                <p className={styles.crateCardDesc}>{t(`arch.${key}`)}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className={`${styles.section} ${styles.ctaSection}`}>
        <div className={styles.container}>
          <div className={styles.ctaBox}>
            <div className={styles.ctaGlow} />
            <h2 className={styles.ctaTitle}>{t('cta.title')}</h2>
            <p className={styles.ctaSub}>{t('cta.sub')}</p>
            <div className={styles.ctaActions}>
              <a href={docsPath} className={styles.btnPrimary}>
                {t('cta.docs')}
              </a>
              <a
                href="https://github.com/bpodwinski/rs-grid"
                target="_blank"
                rel="noopener noreferrer"
                className={styles.btnOutline}
              >
                GitHub ↗
              </a>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
