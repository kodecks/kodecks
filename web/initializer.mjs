export default function myInitializer() {
  return {
    onStart: () => {
      document.getElementById('loading-screen').style.display = 'flex';
    },
    onProgress: ({ current, total }) => {
      const progress = Math.min(Math.round((current / total) * 100), 100);
      document.querySelector('#loading-percentage').textContent = `${progress}%`;
    },
    onComplete: () => {
      document.querySelector('#loading-percentage').textContent = `100%`;
      setTimeout(() => {
        document.getElementById('loading-screen').style.display = 'none';
      }, 100);
    },
    onSuccess: (wasm) => { },
    onFailure: (error) => { }
  }
};