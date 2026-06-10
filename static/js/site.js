document.querySelectorAll('.email-link').forEach(el => {
    el.addEventListener('click', function (e) {
        e.preventDefault();
        window.location = 'mailto:' + this.dataset.user + '@' + this.dataset.domain;
    });
});

document.querySelectorAll('.filter-btn').forEach(btn => {
    btn.addEventListener('click', function () {
        document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
        this.classList.add('active');
        const topic = this.dataset.topic;
        document.querySelectorAll('.article-item').forEach(item => {
            if (topic === 'all' || item.dataset.topic === topic) {
                item.style.display = '';
            } else {
                item.style.display = 'none';
            }
        });
    });
});
