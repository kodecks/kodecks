const estimatedSize = 32850000;

export default function myInitializer () {
  return {
    onStart: () => {
      document.getElementById('loading-screen').style.display = 'flex';
    },
    onProgress: ({current, total}) => {
      const totalSize = total || estimatedSize;
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