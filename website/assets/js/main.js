/**
 * NovaType Website - Main JavaScript
 */

// Navbar scroll effect
const navbar = document.getElementById('navbar');

if (navbar) {
  window.addEventListener('scroll', () => {
    if (window.scrollY > 50) {
      navbar.classList.add('scrolled');
    } else {
      navbar.classList.remove('scrolled');
    }
  });
}

// Mobile navigation toggle
const navToggle = document.getElementById('navToggle');
const navLinks = document.getElementById('navLinks');

if (navToggle && navLinks) {
  navToggle.addEventListener('click', () => {
    navLinks.classList.toggle('active');
  });

  // Close menu when clicking outside
  document.addEventListener('click', (e) => {
    if (!navToggle.contains(e.target) && !navLinks.contains(e.target)) {
      navLinks.classList.remove('active');
    }
  });
}

// Code tabs functionality
const codeTabs = document.querySelectorAll('.code-tab');
const codeContents = document.querySelectorAll('.code-content pre');

if (codeTabs.length > 0) {
  codeTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const tabId = tab.dataset.tab;

      // Update active tab
      codeTabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      // Show corresponding content
      codeContents.forEach(content => {
        if (content.id === `code-${tabId}`) {
          content.classList.remove('hidden');
        } else {
          content.classList.add('hidden');
        }
      });
    });
  });
}

// Smooth scroll for anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
  anchor.addEventListener('click', function (e) {
    e.preventDefault();
    const target = document.querySelector(this.getAttribute('href'));
    if (target) {
      target.scrollIntoView({
        behavior: 'smooth',
        block: 'start'
      });
    }
  });
});

// Copy code button functionality
document.querySelectorAll('pre code').forEach(codeBlock => {
  const pre = codeBlock.parentElement;

  // Create copy button
  const copyBtn = document.createElement('button');
  copyBtn.className = 'copy-btn';
  copyBtn.innerHTML = `
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
      <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"></path>
    </svg>
  `;
  copyBtn.title = 'Copier le code';

  // Style the button
  copyBtn.style.cssText = `
    position: absolute;
    top: 8px;
    right: 8px;
    background: rgba(255,255,255,0.1);
    border: none;
    border-radius: 4px;
    padding: 6px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background 0.2s;
    color: #94a3b8;
  `;

  // Make pre relative for positioning
  pre.style.position = 'relative';

  // Show/hide on hover
  pre.addEventListener('mouseenter', () => {
    copyBtn.style.opacity = '1';
  });
  pre.addEventListener('mouseleave', () => {
    copyBtn.style.opacity = '0';
  });

  // Copy functionality
  copyBtn.addEventListener('click', async () => {
    const code = codeBlock.textContent;
    try {
      await navigator.clipboard.writeText(code);
      copyBtn.innerHTML = `
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 6L9 17l-5-5"></path>
        </svg>
      `;
      copyBtn.style.color = '#10b981';
      setTimeout(() => {
        copyBtn.innerHTML = `
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"></path>
          </svg>
        `;
        copyBtn.style.color = '#94a3b8';
      }, 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  });

  pre.appendChild(copyBtn);
});

// Animate elements on scroll
const observerOptions = {
  threshold: 0.1,
  rootMargin: '0px 0px -50px 0px'
};

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.classList.add('animate-fade-in');
      observer.unobserve(entry.target);
    }
  });
}, observerOptions);

document.querySelectorAll('.feature-card, .card, .demo-card').forEach(el => {
  el.style.opacity = '0';
  observer.observe(el);
});

// Active navigation link highlighting
const currentPath = window.location.pathname;
document.querySelectorAll('.nav-links a, .docs-nav-link').forEach(link => {
  if (link.getAttribute('href') && currentPath.includes(link.getAttribute('href').replace('../', ''))) {
    link.classList.add('active');
  }
});

// Table of contents scroll spy (for docs pages)
const tocLinks = document.querySelectorAll('.docs-nav-link');
const sections = document.querySelectorAll('h2[id], h3[id]');

if (tocLinks.length > 0 && sections.length > 0) {
  const tocObserver = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const id = entry.target.getAttribute('id');
        tocLinks.forEach(link => {
          if (link.getAttribute('href') === `#${id}`) {
            tocLinks.forEach(l => l.classList.remove('active'));
            link.classList.add('active');
          }
        });
      }
    });
  }, {
    threshold: 0.5,
    rootMargin: '-100px 0px -66% 0px'
  });

  sections.forEach(section => tocObserver.observe(section));
}

// Search functionality placeholder
const searchInput = document.getElementById('searchInput');
if (searchInput) {
  searchInput.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    // Implement search functionality here
    console.log('Search query:', query);
  });
}

// Theme toggle (if needed in future)
function toggleTheme() {
  document.body.classList.toggle('dark-theme');
  const isDark = document.body.classList.contains('dark-theme');
  localStorage.setItem('theme', isDark ? 'dark' : 'light');
}

// Load saved theme
const savedTheme = localStorage.getItem('theme');
if (savedTheme === 'dark') {
  document.body.classList.add('dark-theme');
}

// Console easter egg
console.log(`
%c NovaType %c Modern Typesetting System

%cBuilt with passion for beautiful documents.
%chttps://github.com/novatype

`,
  'background: #2563eb; color: white; padding: 5px 10px; border-radius: 4px; font-weight: bold;',
  'color: #2563eb; font-weight: bold;',
  'color: #64748b;',
  'color: #2563eb;'
);
