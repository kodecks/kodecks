export default function myInitializer () {
  return {
    onStart: () => {
      document.getElementById('loading-screen').style.display = 'flex';
    },
    onProgress: ({current, total}) => {
      const totalSize = total || 57600000;
      const progress = Math.min(Math.round((current / totalSize) * 100), 100);
      document.querySelector('#loading-percentage').textContent = `${progress}%`;
    },
    onComplete: () => {
      document.getElementById('loading-screen').style.display = 'none';
    },
    onSuccess: (wasm) => {},
    onFailure: (error) => {}
  }
};