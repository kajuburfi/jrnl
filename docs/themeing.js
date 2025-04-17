/* Theming */
document.addEventListener('DOMContentLoaded', () => {
  const toggleThemeButton = document.getElementById('themeButton');

  // Check the current theme from localStorage
  const currentTheme = localStorage.getItem('theme') || 'mocha';
  if (currentTheme === 'latte') {
      document.body.classList.add('light-theme');
      document.getElementById('themeButton').innerHTML = "Dark";
  }
  if (currentTheme === 'mocha') {
      document.getElementById('themeButton').innerHTML = "Light";
  }

  toggleThemeButton.addEventListener('click', () => {
      // Toggle the theme
      document.body.classList.toggle('light-theme');

      // Update the theme in localStorage
      if (document.body.classList.contains('light-theme')) {
          localStorage.setItem('theme', 'latte');
          document.getElementById('themeButton').innerHTML = "Dark";
      } else {
          localStorage.setItem('theme', 'mocha');
          document.getElementById('themeButton').innerHTML = "Light";
      }
  });
});
