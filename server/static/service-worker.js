self.addEventListener('push', function(event) {
    let not=event.data.json();
    const promiseChain = self.registration.showNotification(not.title,not.options)
    event.waitUntil(promiseChain);
});
